use std::path::PathBuf;
use tauri::AppHandle;

use crate::services::{attachments, clipboard_attachments};
use crate::utils::images;

/// Saves the image on the clipboard and returns its path, or `None` if there is no image.
#[tauri::command]
pub async fn paste_image(app: AppHandle) -> Result<Option<String>, String> {
    let folder = attachments::screenshots_dir(&app)?;
    tokio::task::spawn_blocking(move || clipboard_attachments::save_image(&folder))
        .await
        .map_err(|e| format!("image task failed: {e}"))?
}

/// A small preview of an image this app saved. Other files are refused.
#[tauri::command]
pub async fn preview_image(app: AppHandle, path: String) -> Result<String, String> {
    let folder = attachments::screenshots_dir(&app)?;
    let path = PathBuf::from(path);
    if !attachments::is_inside(&path, &folder) {
        return Err("not an image added by this app".to_string());
    }

    tokio::task::spawn_blocking(move || images::thumbnail_data_url(&path))
        .await
        .map_err(|e| format!("image task failed: {e}"))?
}

/// The large version of an image, for viewing. Only images this app saved are allowed:
/// pasted ones waiting to be sent, and ones already stored in a chat.
#[tauri::command]
pub async fn view_image(app: AppHandle, path: String) -> Result<String, String> {
    let pasted_folder = attachments::screenshots_dir(&app)?;
    let stored_folder = attachments::storage_dir(&app)?;
    let path = PathBuf::from(path);
    if !attachments::is_inside_any(&path, &[&pasted_folder, &stored_folder]) {
        return Err("not an image added by this app".to_string());
    }

    tokio::task::spawn_blocking(move || images::to_data_url(&path))
        .await
        .map_err(|e| format!("image task failed: {e}"))?
}
