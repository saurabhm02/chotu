use crate::models::chat::HistoryMessage;
use crate::models::stream::{Phase, StreamEvent};
use crate::models::Commands;
use crate::utils::context::build_context;

use super::chat::{invoke_ask_ai, invoke_ask_web, Attachments};
use super::ocr::invoke_extract_text;
use super::screen::invoke_capture_screen;

/// (response text, sources)
pub struct CommandResult {
    pub text: String,
    pub sources: Vec<(String, String)>,
    pub is_error: bool,
    /// Screenshots taken for the AI (`/screen`).
    pub screenshots: Vec<String>,
}

impl CommandResult {
    fn ok(text: String) -> Self {
        Self {
            text,
            sources: Vec::new(),
            is_error: false,
            screenshots: Vec::new(),
        }
    }

    fn error(message: String) -> Self {
        Self {
            text: format!("Error: {message}"),
            sources: Vec::new(),
            is_error: true,
            screenshots: Vec::new(),
        }
    }
}
/// Plain chat, with whatever the user attached to the question.
async fn ask_ai<F>(prompt: String, attachments: Attachments, on_event: F) -> CommandResult
where
    F: FnMut(StreamEvent) + 'static,
{
    match invoke_ask_ai(prompt, attachments, on_event).await {
        Ok(res) => CommandResult::ok(res),
        Err(e) => CommandResult::error(e),
    }
}

fn not_ready(msg: &str) -> CommandResult {
    CommandResult::ok(msg.to_string())
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
    history: Vec<HistoryMessage>,
    mut on_event: F,
) -> CommandResult
where
    F: FnMut(StreamEvent) + 'static,
{
    on_event(StreamEvent::Status(Phase::Capturing));
    let shot = match invoke_capture_screen().await {
        Ok(shot) => shot,
        Err(e) => return CommandResult::error(e),
    };
    log::info!(
        "screenshot {} taken ({}x{})",
        shot.id,
        shot.width,
        shot.height
    );

    on_event(StreamEvent::Status(Phase::ReadingText));
    let screen_text = match invoke_extract_text(shot.image_path).await {
        Ok(text) => text,
        Err(e) => return CommandResult::error(e),
    };

    let attachments = Attachments {
        context: build_context(quote.as_deref(), Some(&screen_text)),
        history,
        ..Default::default()
    };
    on_event(StreamEvent::Status(Phase::Thinking));
    ask_ai(
        question_or(question, "What is on my screen?"),
        attachments,
        on_event,
    )
    .await
}

/// `/screen`: photograph the whole screen and show that picture to the model.
/// This needs a vision model (see `AI_VISION_MODEL`).
async fn ask_about_screen_image<F>(
    question: String,
    quote: Option<String>,
    history: Vec<HistoryMessage>,
    mut on_event: F,
) -> CommandResult
where
    F: FnMut(StreamEvent) + 'static,
{
    on_event(StreamEvent::Status(Phase::Capturing));
    let shot = match invoke_capture_screen().await {
        Ok(shot) => shot,
        Err(e) => return CommandResult::error(e),
    };
    log::info!(
        "screenshot {} taken ({}x{})",
        shot.id,
        shot.width,
        shot.height
    );

    // `image_path` moves into the request below; the chat stores a copy of the file.
    let screenshot = shot.image_path.clone();
    let attachments = Attachments {
        context: build_context(quote.as_deref(), None),
        image_paths: vec![shot.image_path],
        history,
    };
    on_event(StreamEvent::Status(Phase::Captured));
    let mut result = ask_ai(
        question_or(question, "What is on this screen?"),
        attachments,
        on_event,
    )
    .await;
    result.screenshots = vec![screenshot];
    result
}
/// Route a parsed slash-command (or plain text) to the right backend call.
pub async fn run_cmd<F>(
    cmd: Option<Commands>,
    query: String,
    quote: Option<String>,
    history: Vec<HistoryMessage>,
    on_event: F,
) -> CommandResult
where
    F: FnMut(StreamEvent) + 'static,
{
    let quote_only = Attachments {
        context: build_context(quote.as_deref(), None),
        history: history.clone(),
        ..Default::default()
    };

    match cmd {
        None => ask_ai(query, quote_only, on_event).await,
        Some(Commands::Web) => match invoke_ask_web(query, quote, history, on_event).await {
            Ok(res) => CommandResult {
                text: res.ans,
                sources: res.sources,
                is_error: false,
                screenshots: Vec::new(),
            },
            Err(e) => CommandResult::error(e),
        },
        Some(Commands::Explain) => {
            ask_ai(
                format!("Explain the following clearly and concisely:\n\n{query}"),
                quote_only,
                on_event,
            )
            .await
        }
        Some(Commands::Analyze) => ask_about_screen_text(query, quote, history, on_event).await,
        Some(Commands::Screen) => ask_about_screen_image(query, quote, history, on_event).await,
        // `/new` is handled by the chat before it gets here.
        Some(Commands::New) => not_ready("Started a new chat."),
        Some(Commands::History) | Some(Commands::Rename) => {
            not_ready("This command is handled by the chat.")
        }
        Some(Commands::Notes) => not_ready("Notes isn't wired up yet (coming in Phase 8)."),
    }
}
