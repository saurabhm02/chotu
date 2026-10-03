use serde::Serialize;

use super::tauri::invoke;

/// `camelCase` because Tauri maps `imagePath` to `image_path` in Rust.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ExtractInput {
    image_path: String,
}

/// Reads the text in an image file (Apple Vision, on this computer).
pub async fn invoke_extract_text(image_path: String) -> Result<String, String> {
    let args = serde_wasm_bindgen::to_value(&ExtractInput { image_path })
        .map_err(|e| e.to_string())?;

    let res = invoke("extract_text", args)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| format!("{e:?}")))?;

    res.as_string()
        .ok_or_else(|| "Failed to parse the text that was read".to_string())
}
