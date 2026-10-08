#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Commands {
    Web,
    Explain,
    Analyze,
    Screen,
    New,
    History,
    Translate,
    Tldr,
    Bullets,
    Refine,
    Rewrite,
    Extract,
}

impl Commands {
    /// The word typed after the slash, e.g. `Commands::Tldr` -> "tldr".
    /// The backend uses it to find the matching prompt.
    pub fn name(self) -> &'static str {
        match self {
            Commands::Web => "web",
            Commands::Explain => "explain",
            Commands::Analyze => "ss",
            Commands::Screen => "screen",
            Commands::New => "new",
            Commands::History => "history",
            Commands::Translate => "translate",
            Commands::Tldr => "tldr",
            Commands::Bullets => "bullets",
            Commands::Refine => "refine",
            Commands::Rewrite => "rewrite",
            Commands::Extract => "extract",
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Command {
    pub name: &'static str,
    pub description: &'static str,
    pub cmd: Commands,
}

pub fn all_commands() -> Vec<Command> {
    vec![
        Command {
            name: "web",
            description: "Search the web and cite the sources",
            cmd: Commands::Web,
        },
        Command {
            name: "explain",
            description: "Explain an idea or code in plain words, with an example",
            cmd: Commands::Explain,
        },
        Command {
            name: "ss",
            description: "Read the text on your screen, then ask about it",
            cmd: Commands::Analyze,
        },
        Command {
            name: "screen",
            description: "Capture your screen and show it to the AI",
            cmd: Commands::Screen,
        },
        Command {
            name: "new",
            description: "Start a new chat",
            cmd: Commands::New,
        },
        Command {
            name: "history",
            description: "Open an earlier chat",
            cmd: Commands::History,
        },
        Command {
            name: "translate",
            description: "Translate text into another language",
            cmd: Commands::Translate,
        },
        Command {
            name: "tldr",
            description: "Sum up text in 1-3 short sentences",
            cmd: Commands::Tldr,
        },
        Command {
            name: "bullets",
            description: "Pull out the key points as a bullet list",
            cmd: Commands::Bullets,
        },
        Command {
            name: "refine",
            description: "Correct grammar, spelling, and punctuation",
            cmd: Commands::Refine,
        },
        Command {
            name: "rewrite",
            description: "Make text sound natural and casual",
            cmd: Commands::Rewrite,
        },
        Command {
            name: "extract",
            description: "Read all the text in an image or screenshot",
            cmd: Commands::Extract,
        },
    ]
}

pub fn detect_cmd(text: &str) -> (Option<Commands>, String) {
    let trimmed = text.trim();
    for cmd in all_commands() {
        let prefix = format!("/{}", cmd.name);
        if let Some(rest) = trimmed.strip_prefix(&prefix) {
            return (Some(cmd.cmd), rest.trim().to_string());
        }
    }
    // Old `/analysis` name, kept so existing input still works.
    if let Some(rest) = trimmed.strip_prefix("/analysis") {
        return (Some(Commands::Analyze), rest.trim().to_string());
    }
    (None, trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_command_name_matches_the_palette_entry() {
        for command in all_commands() {
            assert_eq!(command.cmd.name(), command.name);
        }
    }

    #[test]
    fn a_message_without_a_command_has_none() {
        let (cmd, rest) = detect_cmd("  just a question ");
        assert_eq!(cmd, None);
        assert_eq!(rest, "just a question");
    }

    #[test]
    fn typing_a_command_finds_it_and_keeps_the_rest() {
        let (cmd, rest) = detect_cmd("/translate hindi good morning");
        assert_eq!(cmd, Some(Commands::Translate));
        assert_eq!(rest, "hindi good morning");
    }
}
