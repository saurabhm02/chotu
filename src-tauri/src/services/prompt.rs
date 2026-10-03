//! Building the text we send to the model.

/// Puts the text the user highlighted together with what they typed.
///
/// - nothing highlighted        -> just the message
/// - highlighted, no message    -> only the highlighted block
/// - highlighted and a message  -> highlighted block, then the request
pub fn with_quote(message: &str, quoted_text: Option<&str>) -> String {
    let quote = quoted_text.map(str::trim).filter(|q| !q.is_empty());

    match quote {
        None => message.to_string(),
        Some(quote) if message.trim().is_empty() => {
            format!("[Highlighted Text]\n\"{quote}\"")
        }
        Some(quote) => {
            format!("[Highlighted Text]\n\"{quote}\"\n\n[Request]\n{message}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_quote_returns_the_message_unchanged() {
        assert_eq!(with_quote("what is rust?", None), "what is rust?");
    }

    #[test]
    fn blank_quote_is_ignored() {
        assert_eq!(with_quote("hi", Some("   ")), "hi");
    }

    #[test]
    fn quote_and_message_are_combined() {
        assert_eq!(
            with_quote("explain this", Some("let x = 1;")),
            "[Highlighted Text]\n\"let x = 1;\"\n\n[Request]\nexplain this"
        );
    }

    #[test]
    fn quote_without_a_message_sends_only_the_quote() {
        assert_eq!(
            with_quote("", Some("let x = 1;")),
            "[Highlighted Text]\n\"let x = 1;\""
        );
    }

    #[test]
    fn quote_is_trimmed() {
        assert_eq!(
            with_quote("why", Some("  hello \n")),
            "[Highlighted Text]\n\"hello\"\n\n[Request]\nwhy"
        );
    }
}
