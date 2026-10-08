use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use super::tauri::{invoke, Channel};
use crate::models::chat::HistoryMessage;
use crate::models::stream::StreamEvent;

#[derive(Clone, Default, Debug)]
pub struct Attachments {
    /// Extra text for the model to read (highlighted text, text from the screen...).
    pub context: Option<String>,
    /// Screenshot files to show the model (`/screen`).
    pub image_paths: Vec<String>,
    pub history: Vec<HistoryMessage>,
}

/// What we send to the backend. `camelCase` because Tauri expects names like
/// `imagePaths` and maps them to `image_paths` in Rust. Empty attachments are left out.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct AskInput {
    query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    context: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    image_paths: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    history: Vec<HistoryMessage>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct WebSearchResponse {
    pub ans: String,
    pub sources: Vec<(String, String)>,
}

#[derive(Serialize, Debug)]
struct AskCommandInput {
    command: String,
    typed: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    selected: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    image_paths: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    history: Vec<HistoryMessage>,
}

/// Call a backend command that streams text chunks through a `Channel`,
/// forwarding each chunk to `on_chunk`, and return the command's final value.
async fn invoke_llm<F>(
    cmd: &str,
    query: String,
    attachments: Attachments,
    on_event: F,
) -> Result<JsValue, String>
where
    F: FnMut(StreamEvent) + 'static,
{
    let input = AskInput {
        query,
        context: attachments.context,
        image_paths: attachments.image_paths,
        history: attachments.history,
    };
    let args = serde_wasm_bindgen::to_value(&input).map_err(|e| e.to_string())?;
    invoke_with_channel(cmd, args, on_event).await
}

async fn invoke_with_channel<F>(
    cmd: &str,
    args: JsValue,
    mut on_event: F,
) -> Result<JsValue, String>
where
    F: FnMut(StreamEvent) + 'static,
{
    let channel = Channel::new();
    let closure =
        Closure::wrap(Box::new(
            move |payload: JsValue| match StreamEvent::from_js(&payload) {
                Some(event) => on_event(event),
                None => log::warn!("unreadable message from the backend: {payload:?}"),
            },
        ) as Box<dyn FnMut(JsValue)>);
    channel.set_onmessage(&closure);

    js_sys::Reflect::set(&args, &"channel".into(), &channel)
        .map_err(|_| "failed to attach channel".to_string())?;

    let res = invoke(cmd, args).await;
    closure.forget();

    res.map_err(|e| e.as_string().unwrap_or_else(|| format!("{e:?}")))
}

pub async fn invoke_ask_ai<F>(
    query: String,
    attachments: Attachments,
    on_event: F,
) -> Result<String, String>
where
    F: FnMut(StreamEvent) + 'static,
{
    let res = invoke_llm("ask_ai", query, attachments, on_event).await?;
    res.as_string()
        .ok_or_else(|| "Failed to parse response string".to_string())
}

pub async fn invoke_ask_command<F>(
    command: &str,
    typed: String,
    selected: Option<String>,
    image_paths: Vec<String>,
    history: Vec<HistoryMessage>,
    on_event: F,
) -> Result<String, String>
where
    F: FnMut(StreamEvent) + 'static,
{
    let input = AskCommandInput {
        command: command.to_string(),
        typed,
        selected,
        image_paths,
        history,
    };
    let args = serde_wasm_bindgen::to_value(&input).map_err(|e| e.to_string())?;

    let res = invoke_with_channel("ask_command", args, on_event).await?;
    res.as_string()
        .ok_or_else(|| "Failed to parse response string".to_string())
}

pub async fn invoke_ask_web<F>(
    query: String,
    selected: Option<String>,
    history: Vec<HistoryMessage>,
    on_event: F,
) -> Result<WebSearchResponse, String>
where
    F: FnMut(StreamEvent) + 'static,
{
    let attachments = Attachments {
        context: selected,
        history,
        ..Default::default()
    };
    let res = invoke_llm("ask_web", query, attachments, on_event).await?;
    serde_wasm_bindgen::from_value(res).map_err(|e| e.to_string())
}
