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
use crate::state::{CompressionType, EncodingType, GATEWAY_STATE, SOCKET};

pub(crate) mod ready;
pub(crate) mod ready_supplemental;

#[derive(Deserialize, Serialize)]
pub enum DispatchTypes {
    READY,
    READY_SUPPLEMENTAL
}

impl From<DispatchTypes> for String {
    fn from(t: DispatchTypes) -> String {
        match t {
            DispatchTypes::READY => String::from("READY"),
            DispatchTypes::READY_SUPPLEMENTAL => String::from("READY_SUPPLEMENTAL")
        }
    }
}


#[derive(Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum DispatchData {
    READY(Box<Ready>),
    READY_SUPPLEMENTAL(ReadySupplemental)
}

pub fn assemble_dispatch(t: DispatchTypes, d: DispatchData) -> GatewayMessage {
    GatewayMessage {
        op: OpCodes::DISPATCH,
        t: Some(String::from(t)),
        s: Some(0),
        d: Some(GatewayData::DISPATCH {
            data: Box::new(d)
        }),
    }
}

pub async fn send_message(message: GatewayMessage) {
    let mut socket_lock = SOCKET.get().lock().await;
    let mut gateway_state_lock = GATEWAY_STATE.get().lock().await;

    let socket = socket_lock.as_mut().unwrap();
    let gateway_state = gateway_state_lock.as_mut().unwrap();

    let mut enforced_zlib = false;

    // Large messages always have to be compressed
    if mem::size_of_val(&message) > 8192 {
        enforced_zlib = true;
    };

    let mut binary = false;

    if gateway_state.compression.is_some() || gateway_state.encoding == EncodingType::Etf || enforced_zlib {
        binary = true;
    }

    socket
        .send(
            match binary {
                true => {
                    Message::Binary({
                        let encoded_message = match gateway_state.encoding {
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
                        } else if let Some(compression_type) = &gateway_state.compression {
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

    drop(socket_lock);
    drop(gateway_state_lock);
}

pub async fn send_close(reason: ErrorCode) {
    let mut socket_lock = SOCKET.get().lock().await;
    let socket = socket_lock.as_mut().unwrap();

    socket.send(Message::Close(
        Some(
            CloseFrame {
                code: reason.into(),
                reason: reason.into()
            }
        )))
        .await
        .expect("Failed to close websocket!");

    drop(socket_lock);
}