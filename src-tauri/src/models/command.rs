#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TextCommand {
    Translate,
    Tldr,
    Bullets,
    Refine,
    Rewrite,
    Explain,
}

impl TextCommand {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "translate" => Some(Self::Translate),
            "tldr" => Some(Self::Tldr),
            "bullets" => Some(Self::Bullets),
            "refine" => Some(Self::Refine),
            "rewrite" => Some(Self::Rewrite),
            "explain" => Some(Self::Explain),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_known_name_gives_its_command() {
        assert_eq!(
            TextCommand::from_name("translate"),
            Some(TextCommand::Translate)
        );
    }
    #[test]
    fn an_unknown_name_gives_nothing() {
        assert_eq!(TextCommand::from_name("web"), None);
        assert_eq!(TextCommand::from_name(""), None);
    }

    #[test]
    fn tldr_is_a_known_name() {
        assert_eq!(TextCommand::from_name("tldr"), Some(TextCommand::Tldr));
    }

    #[test]
    fn every_text_command_is_known_by_its_name() {
        assert_eq!(
            TextCommand::from_name("bullets"),
            Some(TextCommand::Bullets)
        );
        assert_eq!(TextCommand::from_name("refine"), Some(TextCommand::Refine));
        assert_eq!(
            TextCommand::from_name("rewrite"),
            Some(TextCommand::Rewrite)
        );
        assert_eq!(
            TextCommand::from_name("explain"),
            Some(TextCommand::Explain)
        );
    }
}
