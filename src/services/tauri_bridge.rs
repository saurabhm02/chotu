use wasm_bindgen::prelude::*;
use crate::types::chat::AskBarInput;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    pub async fn invoke(cmd: &str, args: JsValue) -> JsValue;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "window"])]
    pub type LogicalSize;

    #[wasm_bindgen(constructor, js_namespace = ["window", "__TAURI__", "window"])]
    pub fn new(width: f64, height: f64) -> LogicalSize;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "window"])]
    pub type TauriWindow;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "window"], js_name = getCurrentWindow)]
    pub fn get_current_window() -> TauriWindow;

    #[wasm_bindgen(method, js_name = setSize)]
    pub fn set_size(this: &TauriWindow, size: LogicalSize) -> js_sys::Promise;

    #[wasm_bindgen(method, js_name = startDragging)]
    pub fn start_dragging(this: &TauriWindow) -> js_sys::Promise;
}

pub async fn resize_window(width: f64, height: f64) -> Result<(), JsValue> {
    let size = LogicalSize::new(width, height);
    let promise = get_current_window().set_size(size);
    wasm_bindgen_futures::JsFuture::from(promise).await?;
    Ok(())
}

pub fn start_window_drag() {
    let _ = get_current_window().start_dragging();
}

pub async fn invoke_ask_ai(prompt: String) -> Result<String, String> {
    let req = AskBarInput { prompt };
    let args = serde_wasm_bindgen::to_value(&req).map_err(|e| e.to_string())?;
    let res = invoke("ask_ai", args).await;
    res.as_string().ok_or_else(|| "Failed to parse response string".to_string())
}
