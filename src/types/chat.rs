use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ChatTurn {
    pub prompt: String,
    pub response: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AskBarInput {
    pub prompt: String,
}
