use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize, Debug)]
pub struct NewMessage {
    pub chat_id: i64,
    pub role: String,
    pub content: String,
    #[serde(default)]
    pub quote: Option<String>,
    #[serde(default)]
    pub sources: Vec<(String, String)>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub is_error: bool,
    #[serde(default)]
    pub elapsed_ms: Option<i64>,
}

/// One row of the history list.
#[derive(Serialize, Debug, PartialEq)]
pub struct ChatSummary {
    pub id: i64,
    pub title: String,
    /// Milliseconds since 1970 of the last message.
    pub updated_at: i64,
}

/// A saved message.
#[derive(Serialize, Debug, PartialEq)]
pub struct StoredMessage {
    pub role: String,
    pub content: String,
    pub quote: Option<String>,
    pub sources: Vec<(String, String)>,
    pub model: Option<String>,
    pub is_error: bool,
    pub elapsed_ms: Option<i64>,
    /// Paths of the stored images (see `services::attachments`).
    pub attachments: Vec<String>,
    pub created_at: i64,
}
