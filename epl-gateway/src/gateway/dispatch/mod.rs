use std::error::Error;
use std::io::Write;
use std::mem;

use axum_tungstenite::Message;

use crate::fragmented_write::two_frame_fragmentaion;
use crate::gateway::schema::channels::ChannelCreate;
use crate::gateway::schema::error_codes::ErrorCode;
use crate::gateway::schema::message::SharedMessage;
use crate::gateway::schema::opcodes::{GatewayData, OpCodes};
use crate::gateway::schema::ready::{Ready, ReadySupplemental};
use crate::gateway::schema::relationships::{RelationshipAdd, RelationshipRemove};
use crate::gateway::schema::GatewayMessage;
use crate::state::{CompressionType, EncodingType, ThreadData};
use flate2::write::ZlibEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use tracing::debug;
use tungstenite::protocol::frame::coding::{CloseCode, Data, OpCode};
use tungstenite::protocol::frame::{CloseFrame, Frame};

pub(crate) mod channel;
pub(crate) mod message;
pub(crate) mod ready;
pub(crate) mod ready_supplemental;
pub(crate) mod relationships;

#[derive(Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum DispatchTypes {
    Ready(Box<Ready>),
    ReadySupplemental(ReadySupplemental),
    RelationshipAdd(RelationshipAdd),
    RelationshipRemove(RelationshipRemove),
    ChannelCreate(ChannelCreate),
    MessageCreate(SharedMessage),
    MessageUpdate(SharedMessage),
}

impl From<DispatchTypes> for String {
    fn from(t: DispatchTypes) -> String {
        match t {
            DispatchTypes::Ready(_) => String::from("READY"),
            DispatchTypes::ReadySupplemental(_) => String::from("READY_SUPPLEMENTAL"),
            DispatchTypes::RelationshipAdd(_) => String::from("RELATIONSHIP_ADD"),
            DispatchTypes::RelationshipRemove(_) => String::from("RELATIONSHIP_REMOVE"),
            DispatchTypes::ChannelCreate(_) => String::from("CHANNEL_CREATE"),
            DispatchTypes::MessageCreate(_) => String::from("MESSAGE_CREATE"),
            DispatchTypes::MessageUpdate(_) => String::from("MESSAGE_UPDATE")
        }
    }
}

pub fn assemble_dispatch(t: DispatchTypes) -> GatewayMessage {
    GatewayMessage {
        op: OpCodes::Dispatch,
        t: Some(String::from(t.clone())),
        s: Some(0),
        d: Some(GatewayData::Dispatch { data: Box::new(t) }),
    }
}

pub async fn send_message(thread_data: &mut ThreadData, message: GatewayMessage) {
    let mut enforced_zlib = false;
    let mut streamed_compression = false;

    // Large messages always have to be compressed
    if mem::size_of_val(&message) > 8192 {
        enforced_zlib = true;
    };

    if thread_data.gateway_state.compression.is_some() {
        streamed_compression = true;
    }

    let encoded_message = match thread_data.gateway_state.encoding {
        EncodingType::Json => {
            debug!("json");
            let message =
                serde_json::to_string(&message.clone()).expect("Failed to encode message as JSON");

            debug!("{}", message);

            message.into_bytes()
        }
        EncodingType::Etf => {
            debug!("etf");
            serde_eetf::to_bytes(&message.clone()).expect("Failed to encode message as ETF")
        }
    };

    match enforced_zlib {
        true => {
            thread_data
                .socket
                .send(Message::Binary({
                    let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
                    e.write_all(&encoded_message)
                        .expect("Failed to compress message!");
                    e.finish().expect("Failed to compress message!")
                }))
                .await
                .expect("Failed to send message to client");
        }
        false => {
            match streamed_compression {
                true => {
                    send_fragments(thread_data, {
                        Message::Binary({
                            let compression_type =
                                thread_data.gateway_state.compression.clone().unwrap();

                            debug!("wegtftr: {:?}", encoded_message);

                            let mut compressed_message = match compression_type {
                                CompressionType::Zlib => {
                                    let mut e =
                                        ZlibEncoder::new(Vec::new(), Compression::default());
                                    e.write_all(&encoded_message)
                                        .expect("Failed to compress message!");
                                    e.finish().expect("Failed to compress message!")
                                }
                                CompressionType::ZlibStreams => {
                                    let mut e =
                                        ZlibEncoder::new(Vec::new(), Compression::default());
                                    e.write_all(&encoded_message)
                                        .expect("Failed to compress message!");
                                    e.finish().expect("Failed to compress message!")
                                }
                                CompressionType::ZstdStreams => {
                                    // TODO: do we uh need this even? no client or bot asks for zstd
                                    todo!()
                                }
                            };

                            compressed_message.append(&mut vec![0u8, 0u8, 255, 255]);

                            compressed_message
                        })
                    })
                    .await
                    .expect("Failed to send message to client");
                }
                false => {
                    // thread_data
                    //     .socket
                    //     .send(
                    //         if thread_data.gateway_state.encoding.eq(&EncodingType::Etf) {
                    //             Message::Binary(encoded_message)
                    //         } else {
                    //             Message::Text(
                    //                 serde_json::to_string(&message)
                    //                     .expect("Failed to encode message as JSON"),
                    //             )
                    //         },
                    //     )
                    //     .await
                    //     .expect("Failed to send message to client");

                    send_fragments(thread_data, {
                        if thread_data.gateway_state.encoding.eq(&EncodingType::Etf) {
                            Message::Binary(encoded_message)
                        } else {
                            Message::Text(
                                serde_json::to_string(&message)
                                    .expect("Failed to encode message as JSON"),
                            )
                        }
                    })
                    .await
                    .expect("Failed to send message to client");
                }
            }
        }
    }
}

pub async fn send_fragments(
    thread_data: &mut ThreadData,
    message: Message,
) -> Result<(), Box<dyn Error + '_>> {
    debug!("erthaddddddddddddddd: {:?}", message);
    let (mut data, opdata) = match message {
        Message::Text(d) => (d.into(), Data::Text),
        Message::Binary(d) => (d, Data::Binary),
        _ => return Ok(()),
    };

    let mut frames = vec![];

    while !data.is_empty() {
        let res: Vec<_> = data.drain(..data.len().min(1024)).collect();
        let frame = Frame::message(res, OpCode::Data(Data::Continue), false);
        frames.push(frame);
    }

    match frames.as_mut_slice() {
        [] => {}
        [first] => {
            let fh = first.header_mut();
            fh.is_final = true;
            fh.opcode = OpCode::Data(opdata);
        }
        [first, second] => {
            two_frame_fragmentaion(first, second, OpCode::Data(opdata));
        }
        [first, .., last] => {
            two_frame_fragmentaion(first, last, OpCode::Data(opdata));
        }
    };

    debug!(
        "Queued fragments: {} ({} bytes pre fragment)",
        frames.len(),
        1024
    );

    for i in frames {
        let message = Message::Frame(i);
        debug!("sending {:?}", message);
        thread_data.socket.send(message).await?;
    }

    Ok(())
}

pub async fn send_close(thread_data: &mut ThreadData, reason: ErrorCode) {
    thread_data
        .socket
        .send(Message::Close(Some(CloseFrame {
            code: CloseCode::Library(reason.into()),
            reason: reason.into(),
        })))
        .await
        .expect("Failed to close websocket!");
}
