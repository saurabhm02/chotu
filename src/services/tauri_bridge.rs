use crate::types::chat::{AskBarInput, AskWebInput, WebSearchResponse};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    pub type Channel;

    #[wasm_bindgen(constructor, js_namespace = ["window", "__TAURI__", "core"])]
    pub fn new() -> Channel;

    #[wasm_bindgen(method, setter, js_name = onmessage)]
    pub fn set_onmessage(this: &Channel, cb: &Closure<dyn FnMut(JsValue)>);

    #[wasm_bindgen(catch, js_namespace = ["window", "__TAURI__", "core"])]
    pub async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;

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

pub async fn invoke_ask_ai<F>(query: String, mut on_chunk: F) -> Result<String, String>
where
    F: FnMut(String) + 'static,
{
    let req = AskBarInput { query };
    let args = serde_wasm_bindgen::to_value(&req).map_err(|e| e.to_string())?;

    let channel = Channel::new();
    let closure = Closure::wrap(Box::new(move |payload: JsValue| {
        if let Some(text) = payload.as_string() {
            on_chunk(text);
        }
    }) as Box<dyn FnMut(JsValue)>);
    channel.set_onmessage(&closure);

    js_sys::Reflect::set(&args, &"channel".into(), &channel)
        .map_err(|_| "failed to attach channel".to_string())?;

    let res = invoke("ask_ai", args).await;
    closure.forget();

    let res = res.map_err(|e| e.as_string().unwrap_or_else(|| format!("{e:?}")))?;
    res.as_string()
        .ok_or_else(|| "Failed to parse response string".to_string())
}

pub async fn invoke_ask_web<F>(query: String, mut on_chunk: F) -> Result<WebSearchResponse, String>
where
    F: FnMut(String) + 'static,
{
    let req = AskWebInput { query };
    let args = serde_wasm_bindgen::to_value(&req).map_err(|e| e.to_string())?;

    let channel = Channel::new();
    let closure = Closure::wrap(Box::new(move |payload: JsValue| {
        if let Some(text) = payload.as_string() {
            on_chunk(text);
        }
    }) as Box<dyn FnMut(JsValue)>);
    channel.set_onmessage(&closure);

    js_sys::Reflect::set(&args, &"channel".into(), &channel)
        .map_err(|_| "failed to attach channel".to_string())?;

    let res = invoke("ask_web", args).await;
    closure.forget();

    let res = res.map_err(|e| e.as_string().unwrap_or_else(|| format!("{e:?}")))?;

    serde_wasm_bindgen::from_value(res).map_err(|e| e.to_string())
}

#[wasm_bindgen(inline_js = r#"
    export function copyTextToClipboard(text) {
        if (navigator.clipboard && navigator.clipboard.writeText) {
            navigator.clipboard.writeText(text);
        } else {
            const textarea = document.createElement('textarea');
            textarea.value = text;
            textarea.style.position = 'fixed';
            textarea.style.opacity = '0';
            document.body.appendChild(textarea);
            textarea.select();
            document.execCommand('copy');
            document.body.removeChild(textarea);
        }
    }
"#)]
extern "C" {
    fn copyTextToClipboard(text: &str);
}

pub fn copy_to_clipboard(text: &str) {
    copyTextToClipboard(text);
}
