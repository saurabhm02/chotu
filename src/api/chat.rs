use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use super::tauri::{invoke, Channel};
use crate::models::stream::StreamEvent;

#[derive(Clone, Default, Debug)]
pub struct Attachments {
    /// Extra text for the model to read (highlighted text, text from the screen...).
    pub context: Option<String>,
    /// Screenshot files to show the model (`/screen`).
    pub image_paths: Vec<String>,
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
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct WebSearchResponse {
    pub ans: String,
    pub sources: Vec<(String, String)>,
}

/// Call a backend command that streams events through a `Channel`,
/// forwarding each event to `on_event`, and return the command's final value.
async fn invoke_llm<F>(
    cmd: &str,
    query: String,
    attachments: Attachments,
    mut on_event: F,
) -> Result<JsValue, String>
where
    F: FnMut(StreamEvent) + 'static,
{
    let input = AskInput {
        query,
        context: attachments.context,
        image_paths: attachments.image_paths,
    };
    let args = serde_wasm_bindgen::to_value(&input).map_err(|e| e.to_string())?;

    let channel = Channel::new();
    let closure = Closure::wrap(Box::new(move |payload: JsValue| {
        match StreamEvent::from_js(&payload) {
            Some(event) => on_event(event),
            None => log::warn!("unreadable message from the backend: {payload:?}"),
        }
    }) as Box<dyn FnMut(JsValue)>);
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

/// `/web`. The highlighted text (if any) travels as `context`: the backend adds
/// its start to the search words and shows all of it to the AI.
pub async fn invoke_ask_web<F>(
    query: String,
    quote: Option<String>,
    on_event: F,
) -> Result<WebSearchResponse, String>
where
    F: FnMut(StreamEvent) + 'static,
{
    let attachments = Attachments {
        context: quote,
        ..Default::default()
    };
    let res = invoke_llm("ask_web", query, attachments, on_event).await?;
    serde_wasm_bindgen::from_value(res).map_err(|e| e.to_string())
}

