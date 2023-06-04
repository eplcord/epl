use std::io::Write;
use std::mem;
use axum::extract::ws::{CloseFrame, Message};
use flate2::Compression;
use flate2::write::ZlibEncoder;
use serde::{Deserialize, Serialize};
use crate::gateway::schema::error_codes::ErrorCode;
use crate::gateway::schema::GatewayMessage;
use crate::gateway::schema::opcodes::{GatewayData, OpCodes};
use crate::gateway::schema::ready::{Ready, ReadySupplemental};
use crate::gateway::schema::relationships::{RelationshipAdd, RelationshipRemove};
use crate::state::{CompressionType, EncodingType, ThreadData};

pub(crate) mod ready;
pub(crate) mod ready_supplemental;
pub(crate) mod relationships;

#[derive(Deserialize, Serialize)]
pub enum DispatchTypes {
    Ready,
    ReadySupplemental,
    RelationshipAdd,
    RelationshipRemove
}

impl From<DispatchTypes> for String {
    fn from(t: DispatchTypes) -> String {
        match t {
            DispatchTypes::Ready => String::from("READY"),
            DispatchTypes::ReadySupplemental => String::from("READY_SUPPLEMENTAL"),
            DispatchTypes::RelationshipAdd => String::from("RELATIONSHIP_ADD"),
            DispatchTypes::RelationshipRemove => String::from("RELATIONSHIP_REMOVE")
        }
    }
}


#[derive(Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum DispatchData {
    Ready(Box<Ready>),
    ReadySupplemental(ReadySupplemental),
    RelationshipAdd(RelationshipAdd),
    RelationshipRemove(RelationshipRemove)
}

pub fn assemble_dispatch(t: DispatchTypes, d: DispatchData) -> GatewayMessage {
    GatewayMessage {
        op: OpCodes::Dispatch,
        t: Some(String::from(t)),
        s: Some(0),
        d: Some(GatewayData::Dispatch {
            data: Box::new(d)
        }),
    }
}

pub async fn send_message(thread_data: &mut ThreadData, message: GatewayMessage) {
    let mut enforced_zlib = false;

    // Large messages always have to be compressed
    if mem::size_of_val(&message) > 8192 {
        enforced_zlib = true;
    };

    let mut binary = false;

    if thread_data.gateway_state.compression.is_some() || thread_data.gateway_state.encoding == EncodingType::Etf || enforced_zlib {
        binary = true;
    }

    thread_data.socket
        .send(
            match binary {
                true => {
                    Message::Binary({
                        let encoded_message = match thread_data.gateway_state.encoding {
                            EncodingType::Json => {
                                let message = serde_json::to_string(&message.clone())
                                    .expect("Failed to encode message as JSON");

                                message.into_bytes()
                            }
                            EncodingType::Etf => {
                                serde_eetf::to_bytes(&message.clone())
                                    .expect("Failed to encode message as ETF")
                            }
                        };

                        if enforced_zlib {
                            let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
                            e.write_all(&encoded_message).expect("Failed to compress message!");
                            e.finish().expect("Failed to compress message!")
                        } else if let Some(compression_type) = &thread_data.gateway_state.compression {
                            match compression_type {
                                CompressionType::Zlib => {
                                    let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
                                    e.write_all(&encoded_message).expect("Failed to compress message!");
                                    e.finish().expect("Failed to compress message!")
                                },
                                CompressionType::ZlibStreams => {
                                    todo!()
                                },
                                CompressionType::ZstdStreams => {
                                    // TODO: do we uh need this even? no client or bot asks for zstd
                                    todo!()
                                }
                            }
                        } else {
                            encoded_message.to_vec()
                        }
                    })
                },
                false => {
                    Message::Text(
                        serde_json::to_string(&message)
                            .expect("Failed to encode message as JSON")
                    )
                }
            }
        )
        .await
        .expect("Failed to send message to client");
}

pub async fn send_close(thread_data: &mut ThreadData, reason: ErrorCode) {
    thread_data.socket.send(Message::Close(
        Some(
            CloseFrame {
                code: reason.into(),
                reason: reason.into()
            }
        )))
        .await
        .expect("Failed to close websocket!");
}