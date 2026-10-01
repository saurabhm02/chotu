use tauri::ipc::Channel;

use crate::ai::client::AiClient;

#[tauri::command]
pub async fn ask_ai(channel: Channel<String>, query: String) -> Result<String, String> {
    AiClient::shared().call_llm(&channel, &query).await
}
