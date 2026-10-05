use serde::Deserialize;

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
