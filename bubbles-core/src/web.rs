use crate::bubble::Bubble;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Request {
    Set(Vec<Bubble>),
    GetInfo,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Response {
    Success,
    Error,
    Bubbles(Vec<Bubble>),
}
