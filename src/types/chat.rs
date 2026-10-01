use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ChatTurn {
    pub prompt: String,
    pub response: String,
    pub sources: Vec<(String, String)>,
    #[serde(default = "current_time_str")]
    pub timestamp: String,
}

pub fn current_time_str() -> String {
    let date = js_sys::Date::new_0();
    let hours = date.get_hours();
    let minutes = date.get_minutes();
    let ampm = if hours >= 12 { "PM" } else { "AM" };
    let h12 = if hours == 0 {
        12
    } else if hours > 12 {
        hours - 12
    } else {
        hours
    };
    format!("{}:{:02} {}", h12, minutes, ampm)
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
