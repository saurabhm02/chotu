use crate::config::{AUTO_WEB_MAX_CHARS, AUTO_WEB_MIN_WORDS};

pub fn is_auto_web_enabled(setting: Option<&str>) -> bool {
    !matches!(
        setting.map(|value| value.trim().to_lowercase()).as_deref(),
        Some("off" | "false" | "no" | "0")
    )
}

pub fn is_worth_asking(question: &str) -> bool {
    let question = question.trim();
    question.split_whitespace().count() >= AUTO_WEB_MIN_WORDS
        && question.chars().count() <= AUTO_WEB_MAX_CHARS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_web_is_on_unless_switched_off() {
        assert!(is_auto_web_enabled(None));
        assert!(is_auto_web_enabled(Some("on")));
        assert!(is_auto_web_enabled(Some("")));
        for value in ["off", "OFF", " false ", "no", "0"] {
            assert!(!is_auto_web_enabled(Some(value)), "{value}");
        }
    }

    #[test]
    fn a_normal_question_is_worth_asking() {
        assert!(is_worth_asking("what is the bitcoin price right now?"));
        assert!(is_worth_asking("  weather today  "));
    }

    #[test]
    fn one_word_or_nothing_is_not() {
        assert!(!is_worth_asking("thanks"));
        assert!(!is_worth_asking("   "));
        assert!(!is_worth_asking(""));
    }

    #[test]
    fn a_pasted_document_is_not() {
        let long = "word ".repeat(200);
        assert!(!is_worth_asking(&long));
    }

    #[test]
    fn the_length_limit_counts_characters_not_bytes() {
        let hindi = "नमस्ते दुनिया ".repeat(20);
        assert!(hindi.len() > AUTO_WEB_MAX_CHARS);
        assert!(hindi.chars().count() <= AUTO_WEB_MAX_CHARS);
        assert!(is_worth_asking(&hindi));
    }
}
