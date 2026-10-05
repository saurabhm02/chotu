use serde::Serialize;

use super::tauri::invoke;
use crate::models::chat::NewMessage;

fn error_text(e: wasm_bindgen::JsValue) -> String {
    e.as_string().unwrap_or_else(|| format!("{e:?}"))
}
pub async fn invoke_chat_create(msg: &str) -> Result<i64, String> {
    let args = js_sys::Object::new();
    js_sys::Reflect::set(&args, &"firstMessage".into(), &msg.into())
        .map_err(|_| "failed to build the arguments".to_string())?;
    let id = invoke("chat_create", args.into())
        .await
        .map_err(error_text)?;
    id.as_f64()
        .map(|num| num as i64)
        .ok_or_else(|| "the chat id was not a number".to_string())
}

pub async fn invoke_chat_add_message(msg: &NewMessage) -> Result<(), String> {
    #[derive(Serialize)]
    struct Args<'a> {
        msg: &'a NewMessage,
    }

    let args = serde_wasm_bindgen::to_value(&Args { msg }).map_err(|e| e.to_string())?;
    invoke("chat_add_message", args)
        .await
        .map(|_| ())
        .map_err(error_text)
}
