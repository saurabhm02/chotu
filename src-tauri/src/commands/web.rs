use tauri::ipc::Channel;

use crate::models::web::WebSearchResponse;
use crate::services::web_answer;

#[tauri::command]
pub async fn ask_web(channel: Channel<String>, query: String) -> Result<WebSearchResponse, String> {
    web_answer::answer(&channel, &query).await
}
