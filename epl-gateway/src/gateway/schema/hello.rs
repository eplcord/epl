use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Hello {
    pub(crate) heartbeat_interval: i32,
}
