use serde::Serialize;

/// for example `{"type":"Token","data":"Hello"}`.
#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum StreamEvent {
    /// What the app is doing right now.
    Status(Phase),
    /// The web pages the answer is based on, numbered from 1 in this order.
    Sources(Vec<Source>),
    /// A piece of the answer text.
    Token(String),
    /// The model that is writing the answer.
    Model(String),
}

#[derive(Serialize, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    /// `/ss` and `/screen`: taking the screenshot.
    Capturing,
    /// `/ss`: reading the words on the screenshot.
    ReadingText,
    Searching,
    Reading,
    Thinking,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Source {
    pub title: String,
    pub url: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn json(event: &StreamEvent) -> String {
        serde_json::to_string(event).unwrap()
    }

    #[test]
    fn token_has_the_expected_shape() {
        assert_eq!(
            json(&StreamEvent::Token("Hi".into())),
            r#"{"type":"Token","data":"Hi"}"#
        );
    }

    #[test]
    fn status_is_a_lowercase_word() {
        assert_eq!(
            json(&StreamEvent::Status(Phase::Thinking)),
            r#"{"type":"Status","data":"thinking"}"#
        );
        assert_eq!(
            json(&StreamEvent::Status(Phase::Searching)),
            r#"{"type":"Status","data":"searching"}"#
        );
    }

    #[test]
    fn two_word_status_uses_an_underscore() {
        assert_eq!(
            json(&StreamEvent::Status(Phase::ReadingText)),
            r#"{"type":"Status","data":"reading_text"}"#
        );
        assert_eq!(
            json(&StreamEvent::Status(Phase::Capturing)),
            r#"{"type":"Status","data":"capturing"}"#
        );
    }

    #[test]
    fn sources_are_a_list_of_title_and_url() {
        let event = StreamEvent::Sources(vec![Source {
            title: "Rust".into(),
            url: "https://rust-lang.org".into(),
        }]);
        assert_eq!(
            json(&event),
            r#"{"type":"Sources","data":[{"title":"Rust","url":"https://rust-lang.org"}]}"#
        );
    }

    #[test]
    fn model_carries_its_name() {
        assert_eq!(
            json(&StreamEvent::Model("gemma".into())),
            r#"{"type":"Model","data":"gemma"}"#
        );
    }
}
