use serde_derive::{Serialize, Deserialize};

// TODO: stub these for now
#[derive(Serialize, Deserialize)]
pub struct Ready {
    user_settings: ReadyStub,
    notes: Vec<ReadyStub>
}

#[derive(Serialize, Deserialize)]
pub struct ReadySupplemental {
    user_settings: ReadyStub,
    notes: Vec<ReadyStub>
}

#[derive(Serialize, Deserialize)]
pub struct ReadyStub { }