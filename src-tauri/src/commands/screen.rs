use tauri::AppHandle;

use crate::models::screen::ScreenShot;
use crate::services::screen;

/// Take a screenshot of the whole display and save it as a PNG.
#[tauri::command]
pub async fn capture_screen(app: AppHandle) -> Result<ScreenShot, String> {
    screen::capture_display(&app).await
}

/// Let the user drag a box on the screen and save that part as a PNG.
/// Returns `None` if they pressed Esc.
#[tauri::command]
pub async fn capture_region(app: AppHandle) -> Result<Option<ScreenShot>, String> {
    screen::capture_region(&app).await
}

