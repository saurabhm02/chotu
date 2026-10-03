//! Joining the pieces of text a user attaches to a question into one block.

/// Combines the text the user highlighted in another app and the text read from
/// their screen. Each piece gets a short heading so the model knows where it came
/// from. Returns `None` when there is nothing to attach.
pub fn build_context(quote: Option<&str>, screen_text: Option<&str>) -> Option<String> {
    let mut sections = Vec::new();

    if let Some(quote) = non_blank(quote) {
        sections.push(format!("Highlighted text:\n\"{quote}\""));
    }
    if let Some(screen) = non_blank(screen_text) {
        sections.push(format!(
            "Text read from the user's screen (OCR, so it may contain mistakes):\n{screen}"
        ));
    }

    if sections.is_empty() {
        None
    } else {
        Some(sections.join("\n\n"))
    }
}

/// The text without surrounding spaces, or `None` if it is missing or blank.
fn non_blank(text: Option<&str>) -> Option<&str> {
    text.map(str::trim).filter(|t| !t.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_attached_gives_none() {
        assert_eq!(build_context(None, None), None);
        assert_eq!(build_context(Some("  "), Some("")), None);
    }

    #[test]
    fn only_a_quote() {
        assert_eq!(
            build_context(Some("let x = 1;"), None),
            Some("Highlighted text:\n\"let x = 1;\"".to_string())
        );
    }

    #[test]
    fn only_screen_text() {
        let context = build_context(None, Some("fn main() {")).unwrap();
        assert!(context.starts_with("Text read from the user's screen"));
        assert!(context.ends_with("fn main() {"));
    }

    #[test]
    fn quote_comes_before_screen_text() {
        let context = build_context(Some("a"), Some("b")).unwrap();
        assert!(
            context.find("Highlighted text").unwrap() < context.find("Text read from").unwrap()
        );
    }
}
