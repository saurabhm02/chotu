use crate::types::Commands;

use super::tauri_bridge::{invoke_ask_ai, invoke_ask_web};

pub type CommandResult = (String, Vec<(String, String)>);

pub async fn run_cmd<F>(cmd: Option<Commands>, query: String, on_chunk: F) -> CommandResult
where
    F: FnMut(String) + 'static,
{
    match cmd {
        None => match invoke_ask_ai(query, on_chunk).await {
            Ok(res) => (res, Vec::new()),
            Err(e) => (format!("Error: {e}"), Vec::new()),
        },
        Some(Commands::Web) => match invoke_ask_web(query, on_chunk).await {
            Ok(res) => (res.ans, res.sources),
            Err(e) => (format!("Error: {e}"), Vec::new()),
        },
        Some(Commands::Notes) => ("Notes isn't wired up yet (coming in Phase 8).".to_string(), Vec::new()),
        Some(Commands::Explain) => {
            let prompt = format!("Explain the following clearly and concisely:\n\n{query}");
            match invoke_ask_ai(prompt, on_chunk).await {
                Ok(res) => (res, Vec::new()),
                Err(e) => (format!("Error: {e}"), Vec::new()),
            }
        }
        Some(Commands::Analyze) => {
            let prompt = format!("Provide an in-depth analysis of the following:\n\n{query}");
            match invoke_ask_ai(prompt, on_chunk).await {
                Ok(res) => (res, Vec::new()),
                Err(e) => (format!("Error: {e}"), Vec::new()),
            }
        }
        Some(Commands::Screen) => ("Screen capture isn't wired up yet (coming in Phase 6).".to_string(), Vec::new()),
    }
}
