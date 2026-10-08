use crate::models::Commands;

pub const DEFAULT_QUERY: &str = "What is in this image?";

pub fn attachments_to_send(cmd: Option<Commands>, attachments: &[String]) -> Vec<String> {
    match cmd {
        None
        | Some(
            Commands::Explain
            | Commands::Translate
            | Commands::Tldr
            | Commands::Bullets
            | Commands::Refine
            | Commands::Rewrite
            | Commands::Extract,
        ) => attachments.to_vec(),
        _ => Vec::new(),
    }
}

pub fn extract_query_or_default(query: &str, has_attachments: bool) -> String {
    if query.trim().is_empty() && has_attachments {
        DEFAULT_QUERY.to_string()
    } else {
        query.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pasted() -> Vec<String> {
        vec!["a.png".to_string(), "b.png".to_string()]
    }

    #[test]
    fn plain_questions_get_the_attachments() {
        assert_eq!(attachments_to_send(None, &pasted()), pasted());
    }

    #[test]
    fn every_text_command_gets_the_attachments() {
        for command in [
            Commands::Explain,
            Commands::Translate,
            Commands::Tldr,
            Commands::Bullets,
            Commands::Refine,
            Commands::Rewrite,
            Commands::Extract,
        ] {
            let result = attachments_to_send(Some(command), &pasted());
            assert_eq!(result, pasted(), "{command:?}");
        }
    }

    #[test]
    fn commands_that_ignore_attachments_get_none() {
        for command in [
            Commands::Web,
            Commands::Screen,
            Commands::Analyze,
            Commands::New,
        ] {
            assert!(attachments_to_send(Some(command), &pasted()).is_empty());
        }
    }

    #[test]
    fn no_pasted_attachments_gives_none() {
        assert!(attachments_to_send(None, &[]).is_empty());
    }

    #[test]
    fn empty_question_with_attachments_gets_the_default() {
        assert_eq!(extract_query_or_default("  ", true), DEFAULT_QUERY)
    }

    #[test]
    fn typed_question_is_kept() {
        assert_eq!(
            extract_query_or_default("what is this?", true),
            "what is this?"
        );
    }

    #[test]
    fn empty_question_without_attachments_stays_empty() {
        assert_eq!(extract_query_or_default("", false), "");
    }
}
