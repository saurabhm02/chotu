use serde::{Deserialize, Serialize};

use crate::models::stream::Phase;
use crate::utils::time::current_time_str;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ChatTurn {
    /// user query.
    pub prompt: String,
    /// Text the user had highlighted, if any.
    #[serde(default)]
    pub quote: Option<String>,
    pub response: String,
    pub sources: Vec<(String, String)>,
    /// What the backend is doing right now (`/web` only). Not saved.
    #[serde(skip)]
    pub status: Option<Phase>,
    /// The model that answered. Not saved.
    #[serde(skip)]
    pub model: String,
    /// How long the answer took, in milliseconds. Not saved.
    #[serde(skip)]
    pub elapsed_ms: Option<u64>,

    #[serde(skip)]
    pub is_error: bool,
    #[serde(default = "current_time_str")]
    pub timestamp: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct HistoryMessage {
    pub role: String,
    pub content: String,
}

#[derive(Clone, PartialEq, Debug, Serialize)]
pub struct NewMessage {
    pub chat_id: i64,
    pub role: String,
    pub content: String,
    pub quote: Option<String>,
    pub sources: Vec<(String, String)>,
    pub model: Option<String>,
    pub is_error: bool,
    pub elapsed_ms: Option<u64>,
}
