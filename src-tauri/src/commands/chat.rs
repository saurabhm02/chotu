use tauri::ipc::Channel;

use crate::services::llm::AiClient;

/// Plain chat. Besides the question it can carry:
/// - `context`: extra text for the model to read (highlighted text, text from the screen)
/// - `image_paths`: pictures to show the model
/// The frontend sends these as `context` and `imagePaths`.
#[tauri::command]
pub async fn ask_ai(
    channel: Channel<String>,
    query: String,
    context: Option<String>,
    image_paths: Option<Vec<String>>,
) -> Result<String, String> {
    AiClient::shared()
        .call_llm(
            &channel,
            &query,
            context.as_deref(),
            &image_paths.unwrap_or_default(),
        )
        .await
}

