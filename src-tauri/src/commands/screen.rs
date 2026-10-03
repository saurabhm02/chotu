use tauri::AppHandle;

use crate::models::screen::ScreenShot;
use crate::services::screen;

/// Take a screenshot of the display and save it as a PNG.
#[tauri::command]
pub async fn capture_screen(app: AppHandle) -> Result<ScreenShot, String> {
    screen::capture_display(&app).await
}
