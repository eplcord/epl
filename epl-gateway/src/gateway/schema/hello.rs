use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Hello {
    pub(crate) heartbeat_interval: i32
}