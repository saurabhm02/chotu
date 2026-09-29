#[derive(Clone, PartialEq, Debug)]
pub enum Commands {
    Web,
    Notes,
    Analysis,
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
            description: "search from web!",
            cmd: Commands::Web,
        },
        Command {
            name: "notes",
            description: "add notes in the notebook!",
            cmd: Commands::Notes,
        },
        Command {
            name: "analysis",
            description: "analysis the input!",
            cmd: Commands::Analysis,
        },
    ]
}
