use serde::Serialize;

use super::tauri::invoke;

#[derive(Serialize)]
struct AutoWebRequest<'a> {
    question: &'a str,
}

pub async fn invoke_should_search_web(question: &str) -> Result<bool, String> {
    let args =
        serde_wasm_bindgen::to_value(&AutoWebRequest { question }).map_err(|e| e.to_string())?;

    let reply = invoke("should_search_web", args)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| format!("{e:?}")))?;

    reply
        .as_bool()
        .ok_or_else(|| "Failed to parse the reply".to_string())
}
