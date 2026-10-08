use crate::config::NO_TEXT_MESSAGE;
use crate::models::chat::ChatMessage;
use crate::models::command::TextCommand;
use crate::models::stream::StreamEvent;
use crate::services::llm::AiClient;
use crate::utils::command_prompt::build_prompt;
use tauri::ipc::Channel;

/// Plain chat. Besides the question it can carry:
/// - `context`: extra text for the model to read (highlighted text, text from the screen)
/// - `image_paths`: pictures to show the model
/// The frontend sends these as `context` and `imagePaths`.
#[tauri::command]
pub async fn ask_ai(
    channel: Channel<StreamEvent>,
    query: String,
    context: Option<String>,
    image_paths: Option<Vec<String>>,
    history: Option<Vec<ChatMessage>>,
) -> Result<String, String> {
    AiClient::shared()
        .call_llm(
            &channel,
            &query,
            context.as_deref(),
            &history.unwrap_or_default(),
            &image_paths.unwrap_or_default(),
        )
        .await
}

/// A text command such as `/translate`. The frontend sends the command's name, what the user
/// typed after it, and the text they had highlighted. The prompt is built here from the
/// command's template, so all prompts live in the backend.
#[tauri::command]
pub async fn ask_command(
    channel: Channel<StreamEvent>,
    command: String,
    typed: String,
    selected: Option<String>,
    image_paths: Option<Vec<String>>,
    history: Option<Vec<ChatMessage>>,
) -> Result<String, String> {
    let image_paths = image_paths.unwrap_or_default();
    let cmd =
        TextCommand::from_name(&command).ok_or_else(|| format!("unknown command: {command}"))?;

    // Nothing to work on: answer with a hint and do not call the model.
    let Some(prompt) = build_prompt(cmd, &typed, selected.as_deref(), !image_paths.is_empty())
    else {
        return Ok(NO_TEXT_MESSAGE.to_string());
    };

    AiClient::shared()
        .call_llm(
            &channel,
            &prompt,
            None,
            &history.unwrap_or_default(),
            &image_paths,
        )
        .await
}
