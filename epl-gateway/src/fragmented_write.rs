// From https://github.com/Voronar/tokio-tungstenite/blob/message_raw_frame_example/examples/fragment_server.rs

use tungstenite::protocol::frame::{
    coding::{Data, OpCode},
    Frame,
};

pub fn two_frame_fragmentaion(first: &mut Frame, second: &mut Frame, first_opdata: OpCode) {
    let fh = first.header_mut();
    fh.is_final = false;
    fh.opcode = first_opdata;

    let sh = second.header_mut();
    sh.is_final = true;
    sh.opcode = OpCode::Data(Data::Continue);
}
