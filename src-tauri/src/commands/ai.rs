use crate::ai::client::AiClient;

#[tauri::command]
pub async fn ask_ai(query: String) -> Result<String, String> {
    log::info!("ask_ai called with query: {}", query);

    let ai_client = AiClient::new();
    let res = ai_client.call_llm(&query).await.inspect_err(|e| {
        log::error!("call_llm failed: {}", e);
    })?;

    log::info!("ask_ai response: {}", res);

    Ok(res)
}
