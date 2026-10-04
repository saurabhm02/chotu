use super::tauri::invoke;

/// Opens a link in the user's normal browser (through the Tauri opener plugin).
pub fn open_url(url: &str) {
    let args = js_sys::Object::new();
    let _ = js_sys::Reflect::set(&args, &"url".into(), &url.into());
    wasm_bindgen_futures::spawn_local(async move {
        if let Err(e) = invoke("plugin:opener|open_url", args.into()).await {
            log::warn!("could not open link: {e:?}");
        }
    });
}
