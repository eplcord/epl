use std::cmp::max;
use std::collections::HashMap;
use axum::{Extension, Json};
use axum::response::IntoResponse;
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};
use sea_orm::ActiveValue::Set;
use serde_derive::Serialize;
use epl_common::database::entities::april_fools2024;
use epl_common::database::entities::prelude::AprilFools2024;
use epl_common::rustflake;
use crate::AppState;
use crate::authorization_extractor::SessionContext;
use crate::http::v9::routes::aprilfools2024::LootboxItems::{BusterBlade, CutePlushie, DreamHammer, Ocarina, OhhhhhBanana, PowerHelment, Quack, SpeedBoost, WumpShell};

#[derive(Serialize, Copy, Clone)]
#[repr(i64)]
enum LootboxItems {
    BusterBlade = 1214340999644446720,
    CutePlushie = 1214340999644446721,
    WumpShell = 1214340999644446722,
    SpeedBoost = 1214340999644446723,
    Ocarina = 1214340999644446724,
    PowerHelment = 1214340999644446725,
    Quack = 1214340999644446726,
    OhhhhhBanana = 1214340999644446727,
    DreamHammer = 1214340999644446728
}

impl Distribution<LootboxItems> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> LootboxItems {
        match rng.gen_range(0..=8) {
            0 => BusterBlade,
            1 => CutePlushie,
            2 => WumpShell,
            3 => SpeedBoost,
            4 => Ocarina,
            5 => PowerHelment,
            6 => Quack,
            7 => OhhhhhBanana,
            _ => DreamHammer
        }
    }
}

#[derive(Serialize)]
struct GetLootboxesRes {
    opened_items: HashMap<i64, u64>,
    redeemed_prize: bool,
    user_id: String
}

pub async fn get_lootboxes(
    Extension(state): Extension<AppState>,
    Extension(session_context): Extension<SessionContext>,
) -> impl IntoResponse {
    let user_loot = AprilFools2024::find()
        .filter(april_fools2024::Column::User.eq(session_context.user.id))
        .all(&state.conn)
        .await
        .expect("Failed to access database!");

    let opened_items = generate_opened_items(user_loot);

    Json(GetLootboxesRes {
        opened_items,
        redeemed_prize: false,
        user_id: session_context.user.id.to_string(),
    })
}

#[derive(Serialize)]
struct CountLootboxesRes {
    current_count: u64,
    previous_count: u64
}

pub async fn count_lootboxes(
    Extension(state): Extension<AppState>,
    Extension(_session_context): Extension<SessionContext>,
) -> impl IntoResponse {
    let total_loot_count = AprilFools2024::find()
        .count(&state.conn)
        .await
        .expect("Failed to access database!");

    Json(CountLootboxesRes {
        current_count: total_loot_count,
        previous_count: max(total_loot_count, 1000) - 1000,
    })
}

#[derive(Serialize)]
struct OpenLootboxRes {
    opened_item: String,
    user_lootbox_data: GetLootboxesRes
}

pub async fn open_lootbox(
    Extension(state): Extension<AppState>,
    Extension(session_context): Extension<SessionContext>,
) -> impl IntoResponse {
    let new_item: LootboxItems = rand::random();

    let snowflake = rustflake::Snowflake::default().generate();

    april_fools2024::ActiveModel {
        id: Set(snowflake),
        user: Set(session_context.user.id),
        item: Set(new_item as i64),
    }
        .insert(&state.conn)
        .await
        .expect("Failed to access database!");

    let user_loot = AprilFools2024::find()
        .filter(april_fools2024::Column::User.eq(session_context.user.id))
        .all(&state.conn)
        .await
        .expect("Failed to access database!");

    let opened_items = generate_opened_items(user_loot);

    Json(OpenLootboxRes {
        opened_item: (new_item as i64).to_string(),
        user_lootbox_data: GetLootboxesRes {
            opened_items,
            redeemed_prize: false,
            user_id: session_context.user.id.to_string(),
        },
    })
}

pub async fn redeem_prize(
    Extension(state): Extension<AppState>,
    Extension(session_context): Extension<SessionContext>,
) -> impl IntoResponse {
    let user_loot = AprilFools2024::find()
        .filter(april_fools2024::Column::User.eq(session_context.user.id))
        .all(&state.conn)
        .await
        .expect("Failed to access database!");

    let opened_items = generate_opened_items(user_loot);

    // TODO: Implement when avatar decorators are implemented

    Json(GetLootboxesRes {
        opened_items,
        redeemed_prize: true,
        user_id: session_context.user.id.to_string(),
    })
}

fn generate_opened_items(user_loot: Vec<april_fools2024::Model>) -> HashMap<i64, u64> {
    let mut opened_items = HashMap::new();

    for i in user_loot {
        match opened_items.insert(i.item, 1) {
            None => {}
            Some(old_item) => {
                opened_items.insert(i.item, old_item + 1);
            }
        }
    }

    opened_items
}