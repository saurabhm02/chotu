use super::tauri::invoke;
use crate::models::screen::ScreenShot;

/// Screenshot of the whole display.
pub async fn invoke_capture_screen() -> Result<ScreenShot, String> {
    let no_args = js_sys::Object::new();
    let res = invoke("capture_screen", no_args.into())
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| format!("{e:?}")))?;

    serde_wasm_bindgen::from_value(res).map_err(|e| e.to_string())
}
