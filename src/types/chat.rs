use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ChatTurn {
    pub prompt: String,
    pub response: String,
    pub sources: Vec<(String, String)>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AskBarInput {
    pub query: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AskWebInput {
    pub query: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct WebSearchResponse {
    pub ans: String,
    pub sources: Vec<(String, String)>,
}
