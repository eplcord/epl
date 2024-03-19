use std::error::Error;
use std::io::Write;
use std::mem;

use axum_tungstenite::Message;

use crate::fragmented_write::two_frame_fragmentaion;
use crate::gateway::schema::channels::{ChannelCreate, ChannelDelete, ChannelRecipientAdd, ChannelRecipientRemove};
use crate::gateway::schema::error_codes::ErrorCode;
use crate::gateway::schema::message::{MessageDelete, SharedMessage};
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
use crate::gateway::dispatch::typing::TypingStart;
use crate::gateway::dispatch::user_note_update::UserNoteUpdate;

pub(crate) mod channel;
pub(crate) mod message;
pub(crate) mod ready;
pub(crate) mod ready_supplemental;
pub(crate) mod relationships;
pub(crate) mod typing;
pub(crate) mod user_note_update;

#[derive(Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum DispatchTypes {
    Ready(Box<Ready>),
    ReadySupplemental(ReadySupplemental),
    RelationshipAdd(RelationshipAdd),
    RelationshipRemove(RelationshipRemove),
    ChannelCreate(ChannelCreate),
    ChannelDelete(ChannelDelete),
    MessageCreate(SharedMessage),
    MessageUpdate(SharedMessage),
    MessageDelete(MessageDelete),
    TypingStart(TypingStart),
    ChannelRecipientAdd(ChannelRecipientAdd),
    ChannelRecipientRemove(ChannelRecipientRemove),
    UserNoteUpdate(UserNoteUpdate)
}

impl From<DispatchTypes> for String {
    fn from(t: DispatchTypes) -> String {
        match t {
            DispatchTypes::Ready(_) => String::from("READY"),
            DispatchTypes::ReadySupplemental(_) => String::from("READY_SUPPLEMENTAL"),
            DispatchTypes::RelationshipAdd(_) => String::from("RELATIONSHIP_ADD"),
            DispatchTypes::RelationshipRemove(_) => String::from("RELATIONSHIP_REMOVE"),
            DispatchTypes::ChannelCreate(_) => String::from("CHANNEL_CREATE"),
            DispatchTypes::ChannelDelete(_) => String::from("CHANNEL_DELETE"),
            DispatchTypes::MessageCreate(_) => String::from("MESSAGE_CREATE"),
            DispatchTypes::MessageUpdate(_) => String::from("MESSAGE_UPDATE"),
            DispatchTypes::MessageDelete(_) => String::from("MESSAGE_DELETE"),
            DispatchTypes::TypingStart(_) => String::from("TYPING_START"),
            DispatchTypes::ChannelRecipientAdd(_) => String::from("CHANNEL_RECIPIENT_ADD"),
            DispatchTypes::ChannelRecipientRemove(_) => String::from("CHANNEL_RECIPIENT_REMOVE"),
            DispatchTypes::UserNoteUpdate(_) => String::from("USER_NOTE_UPDATE"),
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
            let message =
                serde_json::to_string(&message.clone()).expect("Failed to encode message as JSON");

            message.into_bytes()
        }
        EncodingType::Etf => {
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

                            let compressed_message = match compression_type {
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
                                    let mut compressed_message =
                                        e.finish().expect("Failed to compress message!");

                                    compressed_message.extend([0, 0, 0xFF, 0xFF]);
                                    compressed_message
                                }
                                CompressionType::ZstdStreams => {
                                    // TODO: do we uh need this even? no client or bot asks for zstd
                                    todo!()
                                }
                            };

                            compressed_message
                        })
                    })
                    .await
                    .expect("Failed to send message to client");
                }
                false => {
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
