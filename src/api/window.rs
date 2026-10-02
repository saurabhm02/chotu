use wasm_bindgen::JsValue;

use super::tauri::{get_current_window, invoke, LogicalSize};

/// Resize the window, then ask the backend to slide it back on-screen if the
/// new size pushed it past a screen edge.
pub async fn resize_window(width: f64, height: f64) -> Result<(), JsValue> {
    let size = LogicalSize::new(width, height);
    let promise = get_current_window().set_size(size);
    wasm_bindgen_futures::JsFuture::from(promise).await?;

    // Non-fatal: if this fails the window just stays where it is.
    let args = js_sys::Object::new();
    let _ = js_sys::Reflect::set(&args, &"width".into(), &width.into());
    let _ = js_sys::Reflect::set(&args, &"height".into(), &height.into());
    let _ = invoke("keep_window_on_screen", args.into()).await;
    Ok(())
}

pub fn start_window_drag() {
    let _ = get_current_window().start_dragging();
}
