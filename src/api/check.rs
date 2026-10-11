use super::tauri::invoke;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CheckRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    question: String,
}

pub async fn invoke_check_text(text: Option<String>, question: String) -> Result<String, String> {
    let request = CheckRequest { text, question };

    let args = serde_wasm_bindgen::to_value(&request).map_err(|e| e.to_string())?;
    let reply = invoke("check_text", args)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| format!("{e:?}")))?;

    reply
        .as_string()
        .ok_or_else(|| "Failed to parse the reply".to_string())
}
