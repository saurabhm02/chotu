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
    #[serde(default = "current_time_str")]
    pub timestamp: String,
}
