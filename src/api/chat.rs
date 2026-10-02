use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use super::tauri::{invoke, Channel};

#[derive(Serialize, Deserialize, Debug)]
struct AskInput {
    query: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct WebSearchResponse {
    pub ans: String,
    pub sources: Vec<(String, String)>,
}

/// Call a backend command that streams text chunks through a `Channel`,
/// forwarding each chunk to `on_chunk`, and return the command's final value.
async fn invoke_llm<F>(cmd: &str, query: String, mut on_chunk: F) -> Result<JsValue, String>
where
    F: FnMut(String) + 'static,
{
    let args = serde_wasm_bindgen::to_value(&AskInput { query }).map_err(|e| e.to_string())?;

    let channel = Channel::new();
    let closure = Closure::wrap(Box::new(move |payload: JsValue| {
        if let Some(text) = payload.as_string() {
            on_chunk(text);
        }
    }) as Box<dyn FnMut(JsValue)>);
    channel.set_onmessage(&closure);

    js_sys::Reflect::set(&args, &"channel".into(), &channel)
        .map_err(|_| "failed to attach channel".to_string())?;

    let res = invoke(cmd, args).await;
    closure.forget();

    res.map_err(|e| e.as_string().unwrap_or_else(|| format!("{e:?}")))
}

pub async fn invoke_ask_ai<F>(query: String, on_chunk: F) -> Result<String, String>
where
    F: FnMut(String) + 'static,
{
    let res = invoke_llm("ask_ai", query, on_chunk).await?;
    res.as_string()
        .ok_or_else(|| "Failed to parse response string".to_string())
}

pub async fn invoke_ask_web<F>(query: String, on_chunk: F) -> Result<WebSearchResponse, String>
where
    F: FnMut(String) + 'static,
{
    let res = invoke_llm("ask_web", query, on_chunk).await?;
    serde_wasm_bindgen::from_value(res).map_err(|e| e.to_string())
}
