#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TextCommand {
    Translate,
    Tldr,
}

impl TextCommand {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "translate" => Some(Self::Translate),
            "tldr" => Some(Self::Tldr),
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
}
