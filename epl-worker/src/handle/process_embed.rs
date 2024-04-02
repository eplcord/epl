use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, EntityTrait};
use serde_json::Value;
use tracing::error;
use url::Url;
use epl_common::database::entities::embed;
use epl_common::database::entities::prelude::Message;
use epl_common::{rustflake, URL_REGEX};
use epl_common::nats::{Messages, send_nats_message};
use crate::AppState;

pub async fn process_embed(state: &AppState, message_id: i64) {
    if state.options.mediaproxy_url.is_none() {
        error!("Wanted to get embed but the mediaproxy's url was not provided!")
    }

    let mut snowflake_factory = rustflake::Snowflake::default();

    let message = Message::find_by_id(message_id)
        .one(&state.db)
        .await
        .expect("Failed to access database!")
        .expect("Failed to get message requested by NATS!");

    for i in URL_REGEX.captures_iter(&message.content) {
        let url = Url::parse(i.get(0).unwrap().as_str());

        if url.is_err() {
            continue;
        }

        let captured_url = url.unwrap();

        let call_url = format!("https://{}/embed/{}/{}{}?{}",
            state.options.mediaproxy_url.clone().unwrap(),
            captured_url.scheme(),
            captured_url.host_str().unwrap(),
            captured_url.path(),
            captured_url.query().unwrap_or_default()
        );

        match ureq::get(&call_url).call() {
            Ok(response) => {
                match response.into_json::<Value>() {
                    Ok(response) => {
                        if response.is_null() {
                            continue;
                        }

                        for i in response.as_array().unwrap() {
                            let new_embed = embed::ActiveModel {
                                id: Set(snowflake_factory.generate()),
                                message: Set(message.id),
                                content: Set(i.clone()),
                            };

                            new_embed.insert(&state.db).await.expect("Failed to access database!");
                        }

                        send_nats_message(
                            &state.nats,
                            message.channel_id.to_string(),
                            Messages::MessageUpdate {
                                id: message.id,
                            }
                        ).await;
                    }
                    Err(error) => {
                        error!("Cannot transform output as JSON! {}", error)
                    }
                }
            }
            Err(ureq::Error::Status(code, response)) => {
                error!("Mediaproxy service unavailable! code: {}, response {:#?}", code, response)
            }
            Err(_) => {
                error!("Transport error!")
            }
        }
    }
}