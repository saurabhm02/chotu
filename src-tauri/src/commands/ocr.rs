use std::path::PathBuf;

use crate::services::ocr;

/// Reads the text in an image file (the frontend sends the path as `imagePath`).
#[tauri::command]
pub async fn extract_text(image_path: String) -> Result<String, String> {
    // Vision takes a moment, so run it on its own thread instead of blocking the app.
    let path = PathBuf::from(image_path);
    tokio::task::spawn_blocking(move || ocr::read_text(&path))
        .await
        .map_err(|e| format!("text reading task failed: {e}"))?
}
