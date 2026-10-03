use crate::models::Commands;
use crate::utils::context::build_context;

use super::chat::{invoke_ask_ai, invoke_ask_web, Attachments};
use super::ocr::invoke_extract_text;
use super::screen::invoke_capture_screen;

/// (response text, sources)
pub type CommandResult = (String, Vec<(String, String)>);

/// Plain chat, with whatever the user attached to the question.
async fn ask_ai<F>(prompt: String, attachments: Attachments, on_chunk: F) -> CommandResult
where
    F: FnMut(String) + 'static,
{
    match invoke_ask_ai(prompt, attachments, on_chunk).await {
        Ok(res) => (res, Vec::new()),
        Err(e) => error_result(e),
    }
}

fn error_result(message: String) -> CommandResult {
    (format!("Error: {message}"), Vec::new())
}

fn not_ready(msg: &str) -> CommandResult {
    (msg.to_string(), Vec::new())
}

/// The user's question, or `fallback` when they typed only the command.
fn question_or(question: String, fallback: &str) -> String {
    if question.trim().is_empty() {
        fallback.to_string()
    } else {
        question
    }
}

/// `/ss`: photograph the screen, read the text in it, and ask about that text.
/// Only text reaches the model, so any text model can answer.
async fn ask_about_screen_text<F>(
    question: String,
    quote: Option<String>,
    on_chunk: F,
) -> CommandResult
where
    F: FnMut(String) + 'static,
{
    let shot = match invoke_capture_screen().await {
        Ok(shot) => shot,
        Err(e) => return error_result(e),
    };
    log::info!(
        "screenshot {} taken ({}x{})",
        shot.id,
        shot.width,
        shot.height
    );

    let screen_text = match invoke_extract_text(shot.image_path).await {
        Ok(text) => text,
        Err(e) => return error_result(e),
    };

    let attachments = Attachments {
        context: build_context(quote.as_deref(), Some(&screen_text)),
        ..Default::default()
    };
    ask_ai(
        question_or(question, "What is on my screen?"),
        attachments,
        on_chunk,
    )
    .await
}

/// `/screen`: photograph the whole screen and show that picture to the model.
/// This needs a vision model (see `AI_VISION_MODEL`).
async fn ask_about_screen_image<F>(
    question: String,
    quote: Option<String>,
    on_chunk: F,
) -> CommandResult
where
    F: FnMut(String) + 'static,
{
    let shot = match invoke_capture_screen().await {
        Ok(shot) => shot,
        Err(e) => return error_result(e),
    };
    log::info!(
        "screenshot {} taken ({}x{})",
        shot.id,
        shot.width,
        shot.height
    );

    let attachments = Attachments {
        context: build_context(quote.as_deref(), None),
        image_paths: vec![shot.image_path],
    };
    ask_ai(
        question_or(question, "What is on this screen?"),
        attachments,
        on_chunk,
    )
    .await
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
    let quote_only = Attachments {
        context: build_context(quote.as_deref(), None),
        ..Default::default()
    };

    match cmd {
        None => ask_ai(query, quote_only, on_chunk).await,
        // `/web` ignores highlighted text for now: a long quote makes a bad search.
        Some(Commands::Web) => match invoke_ask_web(query, on_chunk).await {
            Ok(res) => (res.ans, res.sources),
            Err(e) => error_result(e),
        },
        Some(Commands::Explain) => {
            ask_ai(
                format!("Explain the following clearly and concisely:\n\n{query}"),
                quote_only,
                on_chunk,
            )
            .await
        }
        Some(Commands::Analyze) => ask_about_screen_text(query, quote, on_chunk).await,
        Some(Commands::Screen) => ask_about_screen_image(query, quote, on_chunk).await,
        Some(Commands::Notes) => not_ready("Notes isn't wired up yet (coming in Phase 8)."),
    }
}

