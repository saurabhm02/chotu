use tauri::ipc::Channel;

use crate::services::llm::AiClient;

/// Plain chat. `quoted_text` is whatever the user had highlighted in another
/// app (the frontend sends it as `quotedText`).
#[tauri::command]
pub async fn ask_ai(
    channel: Channel<String>,
    query: String,
    quoted_text: Option<String>,
) -> Result<String, String> {
    AiClient::shared()
        .call_llm(&channel, &query, quoted_text.as_deref())
        .await
}
