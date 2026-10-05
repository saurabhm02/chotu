use serde::Serialize;
use wasm_bindgen::JsValue;

use super::tauri::invoke;
use crate::models::chat::{ChatSummary, NewMessage, StoredMessage};

fn error_text(e: JsValue) -> String {
    e.as_string().unwrap_or_else(|| format!("{e:?}"))
}

/// The `{ name: value }` object Tauri passes to a command. Names are camelCase;
/// Tauri maps `chatId` to `chat_id`.
fn args(pairs: &[(&str, JsValue)]) -> Result<JsValue, String> {
    let object = js_sys::Object::new();
    for (name, value) in pairs {
        js_sys::Reflect::set(&object, &JsValue::from_str(name), value)
            .map_err(|_| "failed to build the arguments".to_string())?;
    }
    Ok(object.into())
}

/// Creates a chat from its first message and returns its id.
pub async fn invoke_chat_create(first_message: &str) -> Result<i64, String> {
    let args = args(&[("firstMessage", first_message.into())])?;
    let id = invoke("chat_create", args).await.map_err(error_text)?;
    id.as_f64()
        .map(|number| number as i64)
        .ok_or_else(|| "the chat id was not a number".to_string())
}

/// Saves a message and returns its id.
pub async fn invoke_chat_add_message(message: &NewMessage) -> Result<i64, String> {
    #[derive(Serialize)]
    struct Args<'a> {
        message: &'a NewMessage,
    }
    let args = serde_wasm_bindgen::to_value(&Args { message }).map_err(|e| e.to_string())?;

    let id = invoke("chat_add_message", args).await.map_err(error_text)?;
    id.as_f64()
        .map(|number| number as i64)
        .ok_or_else(|| "the message id was not a number".to_string())
}

pub async fn invoke_chat_list(filter: &str) -> Result<Vec<ChatSummary>, String> {
    let args = args(&[("filter", filter.into())])?;
    let list = invoke("chat_list", args).await.map_err(error_text)?;
    serde_wasm_bindgen::from_value(list).map_err(|e| e.to_string())
}

pub async fn invoke_chat_messages(chat_id: i64) -> Result<Vec<StoredMessage>, String> {
    let args = args(&[("chatId", JsValue::from_f64(chat_id as f64))])?;
    let messages = invoke("chat_messages", args).await.map_err(error_text)?;
    serde_wasm_bindgen::from_value(messages).map_err(|e| e.to_string())
}

pub async fn invoke_chat_rename(chat_id: i64, title: &str) -> Result<String, String> {
    let args = args(&[
        ("chatId", JsValue::from_f64(chat_id as f64)),
        ("title", title.into()),
    ])?;
    let saved = invoke("chat_rename", args).await.map_err(error_text)?;
    saved
        .as_string()
        .ok_or_else(|| "the new name was not text".to_string())
}

pub async fn invoke_chat_delete(chat_id: i64) -> Result<(), String> {
    let args = args(&[("chatId", JsValue::from_f64(chat_id as f64))])?;
    invoke("chat_delete", args)
        .await
        .map(|_| ())
        .map_err(error_text)
}

pub async fn invoke_chat_generate_title(
    chat_id: i64,
    first_message: &str,
    answer: &str,
) -> Result<String, String> {
    let args = args(&[
        ("chatId", JsValue::from_f64(chat_id as f64)),
        ("firstMessage", first_message.into()),
        ("answer", answer.into()),
    ])?;
    let title = invoke("chat_generate_title", args)
        .await
        .map_err(error_text)?;
    title
        .as_string()
        .ok_or_else(|| "the title was not text".to_string())
}

pub async fn invoke_store_attachments(
    message_id: i64,
    screenshots: &[String],
) -> Result<Vec<String>, String> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Args<'a> {
        message_id: i64,
        screenshots: &'a [String],
    }
    let args = serde_wasm_bindgen::to_value(&Args {
        message_id,
        screenshots,
    })
    .map_err(|e| e.to_string())?;

    let stored = invoke("store_attachments", args)
        .await
        .map_err(error_text)?;
    serde_wasm_bindgen::from_value(stored).map_err(|e| e.to_string())
}

pub async fn invoke_attachment_data_url(path: &str) -> Result<String, String> {
    let args = args(&[("path", path.into())])?;
    let url = invoke("attachment_data_url", args)
        .await
        .map_err(error_text)?;
    url.as_string()
        .ok_or_else(|| "the attachment was not text".to_string())
}
