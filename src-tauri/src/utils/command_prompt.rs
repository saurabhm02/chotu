use crate::config::{
    BULLET_PROMPT_TEMPLATE, EXPLAIN_PROMPT_TEMPLATE, REFINE_PROMPT_TEMPLATE,
    REWRITE_PROMPT_TEMPLATE, TLDR_PROMPT_TEMPLATE, TRANSLATE_PROMPT_TEMPLATE,
};
use crate::models::command::TextCommand;

const IMAGE_ONLY_INPUT: &str = "(The text is in the attached image.)";

pub fn build_prompt(
    cmd: TextCommand,
    typed: &str,
    selected: Option<&str>,
    has_images: bool,
) -> Option<String> {
    let input = match join_input(selected, typed) {
        Some(input) => input,
        None if has_images => IMAGE_ONLY_INPUT.to_string(),
        None => return None,
    };

    let template = match cmd {
        TextCommand::Translate => TRANSLATE_PROMPT_TEMPLATE,
        TextCommand::Tldr => TLDR_PROMPT_TEMPLATE,
        TextCommand::Bullets => BULLET_PROMPT_TEMPLATE,
        TextCommand::Refine => REFINE_PROMPT_TEMPLATE,
        TextCommand::Rewrite => REWRITE_PROMPT_TEMPLATE,
        TextCommand::Explain => EXPLAIN_PROMPT_TEMPLATE,
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
        (false, false) => Some(format!("{selected}\n\n[Additional instruction]: {typed}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TRANSLATE: TextCommand = TextCommand::Translate;

    /// Testing only text input now
    fn text_prompt(cmd: TextCommand, typed: &str, selected: Option<&str>) -> Option<String> {
        build_prompt(cmd, typed, selected, false)
    }

    #[test]
    fn each_text_command_uses_its_own_prompt() {
        let bullets = text_prompt(TextCommand::Bullets, "notes", None).unwrap();
        let refine = text_prompt(TextCommand::Refine, "notes", None).unwrap();
        let rewrite = text_prompt(TextCommand::Rewrite, "notes", None).unwrap();
        let explain = text_prompt(TextCommand::Explain, "notes", None).unwrap();

        assert!(bullets.contains("bulleted list"));
        assert!(refine.contains("correcting grammar"));
        assert!(rewrite.contains("Rewrite the text"));
        assert!(explain.contains("plain, simple language"));
    }

    #[test]
    fn every_text_command_ends_with_the_text() {
        for cmd in [
            TextCommand::Translate,
            TextCommand::Tldr,
            TextCommand::Bullets,
            TextCommand::Refine,
            TextCommand::Rewrite,
            TextCommand::Explain,
        ] {
            let prompt = text_prompt(cmd, "my text", None).unwrap();
            assert!(prompt.ends_with("Text: my text"), "{cmd:?}");
            assert!(!prompt.contains("$INPUT"), "{cmd:?}");
        }
    }

    #[test]
    fn an_image_alone_is_enough_to_run_a_command() {
        let prompt = build_prompt(TextCommand::Tldr, "", None, true).unwrap();
        assert!(prompt.ends_with("Text: (The text is in the attached image.)"));
    }

    #[test]
    fn typed_text_wins_over_the_image_placeholder() {
        let prompt = build_prompt(TextCommand::Tldr, "hello", None, true).unwrap();
        assert!(prompt.ends_with("Text: hello"));
    }

    #[test]
    fn highlighted_text_alone_is_the_input() {
        let prompt = text_prompt(TRANSLATE, "", Some("good morning")).unwrap();
        assert!(prompt.ends_with("Text: good morning"));
    }

    #[test]
    fn highlighted_and_typed_text_are_joined_with_a_label() {
        let prompt = text_prompt(TRANSLATE, "hindi", Some("good morning")).unwrap();
        assert!(prompt.ends_with("Text: good morning\n\n[Additional instruction]: hindi"));
    }

    #[test]
    fn no_text_and_no_image_gives_nothing() {
        assert_eq!(build_prompt(TextCommand::Tldr, " ", None, false), None);
    }
}
