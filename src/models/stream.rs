use serde::Deserialize;
use wasm_bindgen::JsValue;

#[derive(Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum StreamEvent {
    Status(Phase),
    Sources(Vec<Source>),
    Token(String),
    Model(String),
}

#[derive(Deserialize, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Capturing,
    Captured,
    ReadingText,
    Preparing,
    Analyzing,
    Searching,
    Reading,
    Thinking,
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct Source {
    pub title: String,
    pub url: String,
}

impl StreamEvent {
    /// Reads one JSON message, or `None` if it isn't a message we know.
    pub fn from_json(json: &str) -> Option<Self> {
        serde_json::from_str(json).ok()
    }

    /// Reads one message as the channel delivers it (a plain JavaScript object).
    pub fn from_js(payload: &JsValue) -> Option<Self> {
        let json = js_sys::JSON::stringify(payload).ok()?.as_string()?;
        Self::from_json(&json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // These strings are exactly what the backend sends (see the backend tests).
    #[test]
    fn reads_a_token() {
        assert_eq!(
            StreamEvent::from_json(r#"{"type":"Token","data":"Hi"}"#),
            Some(StreamEvent::Token("Hi".into()))
        );
    }

    #[test]
    fn reads_a_status() {
        assert_eq!(
            StreamEvent::from_json(r#"{"type":"Status","data":"thinking"}"#),
            Some(StreamEvent::Status(Phase::Thinking))
        );
        assert_eq!(
            StreamEvent::from_json(r#"{"type":"Status","data":"reading"}"#),
            Some(StreamEvent::Status(Phase::Reading))
        );
    }

    #[test]
    fn reads_the_two_word_status() {
        assert_eq!(
            StreamEvent::from_json(r#"{"type":"Status","data":"reading_text"}"#),
            Some(StreamEvent::Status(Phase::ReadingText))
        );
    }

    #[test]
    fn reads_sources() {
        assert_eq!(
            StreamEvent::from_json(
                r#"{"type":"Sources","data":[{"title":"Rust","url":"https://rust-lang.org"}]}"#
            ),
            Some(StreamEvent::Sources(vec![Source {
                title: "Rust".into(),
                url: "https://rust-lang.org".into()
            }]))
        );
    }

    #[test]
    fn reads_the_model_name() {
        assert_eq!(
            StreamEvent::from_json(r#"{"type":"Model","data":"gemma"}"#),
            Some(StreamEvent::Model("gemma".into()))
        );
    }

    #[test]
    fn unknown_messages_are_ignored() {
        assert_eq!(
            StreamEvent::from_json(r#"{"type":"Banana","data":1}"#),
            None
        );
        assert_eq!(StreamEvent::from_json("not json"), None);
    }
}
