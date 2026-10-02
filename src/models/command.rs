#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Commands {
    Web,
    Notes,
    Explain,
    Analyze,
    Screen,
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
            name: "notes",
            description: "Save or search notes",
            cmd: Commands::Notes,
        },
        Command {
            name: "explain",
            description: "Explain code or text",
            cmd: Commands::Explain,
        },
        Command {
            name: "ss",
            description: "Analyze content",
            cmd: Commands::Analyze,
        },
        Command {
            name: "screen",
            description: "Capture screen region",
            cmd: Commands::Screen,
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
