//! Raw bindings to the `window.__TAURI__` JS globals. Nothing else in the
//! frontend should touch these directly; use `api::chat` / `api::window`.
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    pub type Channel;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"])]
    pub fn listen(event: &str, handler: &Closure<dyn FnMut(JsValue)>) -> js_sys::Promise;

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

/// Run `callback` every time the backend emits `name`.
pub fn listen_to_event<F>(name: &'static str, mut callback: F)
where
    F: FnMut() + 'static,
{
    let closure = Closure::wrap(Box::new(move |_: JsValue| {
        callback();
    }) as Box<dyn FnMut(JsValue)>);

    let promise = listen(name, &closure);
    wasm_bindgen_futures::spawn_local(async move {
        let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
        closure.forget();
    });
}

/// Run `callback` with the text payload every time the backend emits `name`.
pub fn listen_to_text_event<F>(name: &'static str, mut callback: F)
where
    F: FnMut(String) + 'static,
{
    let closure = Closure::wrap(Box::new(move |event: JsValue| {
        let payload = js_sys::Reflect::get(&event, &JsValue::from_str("payload")).ok();
        if let Some(text) = payload.and_then(|p| p.as_string()) {
            callback(text);
        }
    }) as Box<dyn FnMut(JsValue)>);

    let promise = listen(name, &closure);
    wasm_bindgen_futures::spawn_local(async move {
        let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
        closure.forget();
    });
}
