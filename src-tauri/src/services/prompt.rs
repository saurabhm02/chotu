//! Building the text we send to the model.

/// Most characters of context we send. A full screen of code is well under this.
const MAX_CONTEXT_CHARS: usize = 12_000;

/// Puts the extra text the user attached (`context`) in front of what they typed.
///
/// - no context              -> just the question
/// - context, no question    -> only the context block
/// - context and a question  -> `[Context]` block, then the `[Request]`
pub fn build_message(request: &str, context: Option<&str>) -> String {
    let context = context.map(str::trim).filter(|c| !c.is_empty());

    match context {
        None => request.to_string(),
        Some(context) if request.trim().is_empty() => {
            format!("[Context]\n{}", shorten(context))
        }
        Some(context) => {
            format!("[Context]\n{}\n\n[Request]\n{request}", shorten(context))
        }
    }
}

/// Cuts very long text down to `MAX_CONTEXT_CHARS` characters.
fn shorten(text: &str) -> String {
    if text.chars().count() <= MAX_CONTEXT_CHARS {
        return text.to_string();
    }
    let kept: String = text.chars().take(MAX_CONTEXT_CHARS).collect();
    format!("{kept}\n...[cut: there was more text]")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_context_returns_the_message_unchanged() {
        assert_eq!(build_message("what is rust?", None), "what is rust?");
    }

    #[test]
    fn blank_context_is_ignored() {
        assert_eq!(build_message("hi", Some("   \n")), "hi");
    }

    #[test]
    fn context_and_message_are_combined() {
        assert_eq!(
            build_message("explain this", Some("let x = 1;")),
            "[Context]\nlet x = 1;\n\n[Request]\nexplain this"
        );
    }

    #[test]
    fn context_without_a_message_sends_only_the_context() {
        assert_eq!(
            build_message("", Some("let x = 1;")),
            "[Context]\nlet x = 1;"
        );
    }

    #[test]
    fn context_is_trimmed() {
        assert_eq!(
            build_message("why", Some("  hello \n")),
            "[Context]\nhello\n\n[Request]\nwhy"
        );
    }

    #[test]
    fn very_long_context_is_cut() {
        let long = "a".repeat(MAX_CONTEXT_CHARS + 500);
        let result = build_message("q", Some(&long));
        assert!(result.contains("[cut: there was more text]"));
        assert!(result.matches('a').count() < long.len());
    }
}

