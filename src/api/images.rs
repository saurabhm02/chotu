use super::tauri::invoke;
use wasm_bindgen::JsValue;

fn error_text(error: JsValue) -> String {
    error.as_string().unwrap_or_else(|| format!("{error:?}"))
}

pub async fn invoke_paste() -> Result<Option<String>, String> {
    let path = invoke("paste_image", js_sys::Object::new().into())
        .await
        .map_err(error_text)?;
    if path.is_null() || path.is_undefined() {
        return Ok(None);
    }

    path.as_string()
        .map(Some)
        .ok_or_else(|| "the image path was not text".to_string())
}

pub async fn invoke_preview(path: &str) -> Result<String, String> {
    let args = js_sys::Object::new();
    js_sys::Reflect::set(&args, &"path".into(), &path.into())
        .map_err(|_| "failed to build the arguments".to_string())?;

    let preview = invoke("preview_image", args.into())
        .await
        .map_err(error_text)?;

    preview
        .as_string()
        .ok_or_else(|| "the preview was not text".to_string())
}

/// The large version of an image, as a `data:` URL for an `<img>`.
pub async fn invoke_view(path: &str) -> Result<String, String> {
    let args = js_sys::Object::new();
    js_sys::Reflect::set(&args, &"path".into(), &path.into())
        .map_err(|_| "failed to build the arguments".to_string())?;

    let image = invoke("view_image", args.into())
        .await
        .map_err(error_text)?;

    image
        .as_string()
        .ok_or_else(|| "the image was not text".to_string())
}
