use crate::config::{TLDR_PROMPT_TEMPLATE, TRANSLATE_PROMPT_TEMPLATE};
use crate::models::command::TextCommand;

pub fn build_prompt(cmd: TextCommand, typed: &str, selected: Option<&str>) -> Option<String> {
    let input = join_input(selected, typed)?;

    let template = match cmd {
        TextCommand::Translate => TRANSLATE_PROMPT_TEMPLATE,
        TextCommand::Tldr => TLDR_PROMPT_TEMPLATE,
    };

    Some(template.trim_end().replace("$INPUT", &input))
}

fn join_input(selected: Option<&str>, typed: &str) -> Option<String> {
    let selected = selected.unwrap_or("").trim();
    let typed = typed.trim();

    match (selected.is_empty(), typed.is_empty()) {
        (true, true) => None,
        (false, true) => Some(selected.to_string()),
        (true, false) => Some(typed.to_string()),
        (false, false) => Some(format!("{selected}\n\n{typed}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TRANSLATE: TextCommand = TextCommand::Translate;

    #[test]
    fn typed_text_goes_into_the_prompt() {
        let prompt = build_prompt(TRANSLATE, "how to run this in code in hinglish", None).unwrap();
        assert!(prompt.ends_with("Text: how to run this in code in hinglish"));
    }

    #[test]
    fn the_language_can_come_first() {
        let prompt = build_prompt(TRANSLATE, "hinglish how to run this in code", None).unwrap();
        assert!(prompt.ends_with("Text: hinglish how to run this in code"));
    }

    #[test]
    fn highlighted_text_goes_into_the_prompt() {
        let prompt = build_prompt(TRANSLATE, "", Some("good morning")).unwrap();
        assert!(prompt.ends_with("Text: good morning"));
    }

    #[test]
    fn highlighted_text_comes_first_and_the_typed_text_after_it() {
        let prompt = build_prompt(TRANSLATE, "hindi", Some("good morning")).unwrap();
        assert!(prompt.ends_with("Text: good morning\n\nhindi"));
    }

    #[test]
    fn the_prompt_says_english_is_the_default() {
        let prompt = build_prompt(TRANSLATE, "hola", None).unwrap();
        assert!(prompt.contains("translate into English"));
    }

    #[test]
    fn no_text_at_all_gives_nothing() {
        assert_eq!(build_prompt(TRANSLATE, "", None), None);
        assert_eq!(build_prompt(TRANSLATE, "   ", Some("  ")), None);
    }

    #[test]
    fn a_dollar_input_inside_the_text_is_kept_as_it_is() {
        let prompt = build_prompt(TRANSLATE, "price is $INPUT", None).unwrap();
        assert!(prompt.ends_with("Text: price is $INPUT"));
    }

    #[test]
    fn tldr_puts_the_typed_text_into_its_own_prompt() {
        let prompt = build_prompt(TextCommand::Tldr, "a very long article", None).unwrap();
        assert!(prompt.contains("TL;DR"));
        assert!(prompt.ends_with("Text: a very long article"));
    }

    #[test]
    fn tldr_summarizes_the_highlighted_text_and_follows_the_typed_instruction() {
        let prompt = build_prompt(TextCommand::Tldr, "in hindi", Some("a long article")).unwrap();
        assert!(prompt.ends_with("Text: a long article\n\nin hindi"));
    }

    #[test]
    fn tldr_with_no_text_gives_nothing() {
        assert_eq!(build_prompt(TextCommand::Tldr, "  ", None), None);
    }
}
