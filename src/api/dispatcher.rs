use crate::models::Commands;

use super::chat::{invoke_ask_ai, invoke_ask_web};
use super::screen::invoke_capture_screen;

/// (response text, sources)
pub type CommandResult = (String, Vec<(String, String)>);

async fn ask_ai<F>(prompt: String, quote: Option<String>, on_chunk: F) -> CommandResult
where
    F: FnMut(String) + 'static,
{
    match invoke_ask_ai(prompt, quote, on_chunk).await {
        Ok(res) => (res, Vec::new()),
        Err(e) => (format!("Error: {e}"), Vec::new()),
    }
}

fn not_ready(msg: &str) -> CommandResult {
    (msg.to_string(), Vec::new())
}

/// Route a parsed slash-command (or plain text) to the right backend call.
pub async fn run_cmd<F>(
    cmd: Option<Commands>,
    query: String,
    quote: Option<String>,
    on_chunk: F,
) -> CommandResult
where
    F: FnMut(String) + 'static,
{
    match cmd {
        None => ask_ai(query, quote, on_chunk).await,
        Some(Commands::Web) => match invoke_ask_web(query, on_chunk).await {
            Ok(res) => (res.ans, res.sources),
            Err(e) => (format!("Error: {e}"), Vec::new()),
        },
        Some(Commands::Explain) => {
            ask_ai(
                format!("Explain the following clearly and concisely:\n\n{query}"),
                quote,
                on_chunk,
            )
            .await
        }
        Some(Commands::Analyze) => match invoke_capture_screen().await {
            Ok(shot) => (
                format!(
                    "Screenshot saved ({}x{}, id {})\n`{}`",
                    shot.width, shot.height, shot.id, shot.image_path
                ),
                Vec::new(),
            ),
            Err(e) => (format!("Error: {e}"), Vec::new()),
        },
        Some(Commands::Notes) => not_ready("Notes isn't wired up yet (coming in Phase 8)."),
        Some(Commands::Screen) => {
            not_ready("Screen capture isn't wired up yet (coming in Phase 8).")
        }
    }
}
