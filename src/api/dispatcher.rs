use crate::models::chat::HistoryMessage;
use crate::models::stream::{Phase, StreamEvent};
use crate::models::Commands;
use crate::utils::context::build_context;
use crate::utils::extract::format_extracted;

use super::chat::{invoke_ask_ai, invoke_ask_command, invoke_ask_web, Attachments};
use super::check::invoke_check_text;
use super::ocr::invoke_extract_text;
use super::screen::invoke_capture_screen;
use super::web::invoke_should_search_web;

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
    selected: Option<String>,
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
        context: build_context(selected.as_deref(), Some(&screen_text)),
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
    selected: Option<String>,
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
        context: build_context(selected.as_deref(), None),
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
    selected: Option<String>,
    history: Vec<HistoryMessage>,
    image_paths: Vec<String>,
    on_event: F,
) -> CommandResult
where
    F: FnMut(StreamEvent) + 'static,
{
    match cmd {
        None => ask_plain_question(query, selected, history, image_paths, on_event).await,
        Some(Commands::Chat) => chat_command(query, selected, history, image_paths, on_event).await,
        Some(Commands::Web) => ask_web_command(query, selected, history, on_event).await,
        Some(
            command @ (Commands::Explain
            | Commands::Translate
            | Commands::Tldr
            | Commands::Bullets
            | Commands::Refine
            | Commands::Rewrite),
        ) => {
            ask_text_command(
                command.name(),
                query,
                selected,
                image_paths,
                history,
                on_event,
            )
            .await
        }
        Some(Commands::Analyze) => ask_about_screen_text(query, selected, history, on_event).await,
        Some(Commands::Screen) => ask_about_screen_image(query, selected, history, on_event).await,
        // `/new` and `/history` never get here: `use_chat` handles them before it calls this
        // function. They are listed only because a `match` must cover every command.
        Some(Commands::New) | Some(Commands::History) => CommandResult::ok(String::new()),
        Some(Commands::Extract) => extract_text_command(image_paths, on_event).await,
        Some(Commands::Check) => check_text_command(query, selected, on_event).await,
    }
}

/// `/translate`, `/tldr`, `/bullets`, `/refine`, `/rewrite`, `/explain`: the backend builds the
/// prompt from the command's name, so this only passes the user's text along.
async fn ask_text_command<F>(
    command: &str,
    typed: String,
    selected: Option<String>,
    image_paths: Vec<String>,
    history: Vec<HistoryMessage>,
    on_event: F,
) -> CommandResult
where
    F: FnMut(StreamEvent) + 'static,
{
    match invoke_ask_command(command, typed, selected, image_paths, history, on_event).await {
        Ok(text) => CommandResult::ok(text),
        Err(e) => CommandResult::error(e),
    }
}

/// `/extract`: read the text in the pasted images (or in a screenshot when none are pasted)
/// and show it as it is. No AI call, so the text is never changed.
async fn extract_text_command<F>(image_paths: Vec<String>, mut on_event: F) -> CommandResult
where
    F: FnMut(StreamEvent) + 'static,
{
    let paths = if image_paths.is_empty() {
        on_event(StreamEvent::Status(Phase::Capturing));
        match invoke_capture_screen().await {
            Ok(shot) => vec![shot.image_path],
            Err(e) => return CommandResult::error(e),
        }
    } else {
        image_paths
    };

    on_event(StreamEvent::Status(Phase::ReadingText));
    let mut texts = Vec::new();
    for path in paths {
        match invoke_extract_text(path).await {
            Ok(text) => texts.push(text),
            Err(e) => log::warn!("could not read the text in an image: {e}"),
        }
    }
    CommandResult::ok(format_extracted(&texts))
}

async fn check_text_command<F>(
    question: String,
    text: Option<String>,
    mut on_event: F,
) -> CommandResult
where
    F: FnMut(StreamEvent) + 'static,
{
    on_event(StreamEvent::Status(Phase::Thinking));
    match invoke_check_text(text, question).await {
        Ok(text) => CommandResult::ok(text),
        Err(e) => CommandResult::error(e),
    }
}

async fn ask_web_command<F>(
    question: String,
    selected: Option<String>,
    history: Vec<HistoryMessage>,
    on_event: F,
) -> CommandResult
where
    F: FnMut(StreamEvent) + 'static,
{
    match invoke_ask_web(question, selected, history, on_event).await {
        Ok(res) => CommandResult {
            text: res.ans,
            sources: res.sources,
            is_error: false,
            screenshots: Vec::new(),
        },
        Err(e) => CommandResult::error(e),
    }
}

fn may_search_automatically(
    question: &str,
    selected: Option<&str>,
    image_paths: &[String],
) -> bool {
    let has_selected_text = selected.is_some_and(|text| !text.trim().is_empty());
    !question.trim().is_empty() && !has_selected_text && image_paths.is_empty()
}

async fn ask_plain_question<F>(
    question: String,
    selected: Option<String>,
    history: Vec<HistoryMessage>,
    image_paths: Vec<String>,
    mut on_event: F,
) -> CommandResult
where
    F: FnMut(StreamEvent) + 'static,
{
    if may_search_automatically(&question, selected.as_deref(), &image_paths) {
        on_event(StreamEvent::Status(Phase::Deciding));
        match invoke_should_search_web(&question).await {
            Ok(true) => return ask_web_command(question, None, history, on_event).await,
            Ok(false) => {}
            Err(e) => log::warn!("could not decide about a web search, answering directly: {e}"),
        }
        on_event(StreamEvent::ClearStatus);
    }
    chat_command(question, selected, history, image_paths, on_event).await
}

async fn chat_command<F>(
    question: String,
    selected: Option<String>,
    history: Vec<HistoryMessage>,
    image_paths: Vec<String>,
    on_event: F,
) -> CommandResult
where
    F: FnMut(StreamEvent) + 'static,
{
    let attachments = Attachments {
        context: build_context(selected.as_deref(), None),
        image_paths,
        history,
    };
    ask_ai(question, attachments, on_event).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_plain_question_may_be_searched() {
        assert!(may_search_automatically("bitcoin price today", None, &[]));
    }

    #[test]
    fn highlighted_text_stops_the_automatic_search() {
        assert!(!may_search_automatically(
            "what does this mean?",
            Some("some highlighted text"),
            &[]
        ));
    }

    #[test]
    fn blank_highlighted_text_does_not_count() {
        assert!(may_search_automatically(
            "bitcoin price today",
            Some("  \n "),
            &[]
        ));
    }

    #[test]
    fn a_pasted_image_stops_the_automatic_search() {
        assert!(!may_search_automatically(
            "what is this?",
            None,
            &["a.png".to_string()]
        ));
    }

    #[test]
    fn an_empty_question_is_not_searched() {
        assert!(!may_search_automatically("   ", None, &[]));
    }
}
