use serde::{Deserialize, Serialize};

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
    #[serde(default = "current_time_str")]
    pub timestamp: String,
}
