use tauri::ipc::Channel;

use crate::models::chat::ChatMessage;
use crate::models::stream::StreamEvent;
use crate::models::web::WebSearchResponse;
use crate::services::web_answer;
#[tauri::command]
pub async fn ask_web(
    channel: Channel<StreamEvent>,
    query: String,
    context: Option<String>,
    history: Option<Vec<ChatMessage>>,
) -> Result<WebSearchResponse, String> {
    web_answer::answer(
        &channel,
        &query,
        context.as_deref(),
        &history.unwrap_or_default(),
    )
    .await
}
