#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Commands {
    Web,
    Explain,
    Analyze,
    Screen,
    New,
    History,
    Translate,
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
            description: "Search the web",
            cmd: Commands::Web,
        },
        Command {
            name: "explain",
            description: "Explain code or text",
            cmd: Commands::Explain,
        },
        Command {
            name: "ss",
            description: "Read text from your screen",
            cmd: Commands::Analyze,
        },
        Command {
            name: "screen",
            description: "Show your screen to the AI",
            cmd: Commands::Screen,
        },
        Command {
            name: "new",
            description: "start a new chat",
            cmd: Commands::New,
        },
        Command {
            name: "history",
            description: "open an earlier chat",
            cmd: Commands::History,
        },
        Command {
            name: "translate",
            description: "translate text to another language",
            cmd: Commands::Translate,
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
