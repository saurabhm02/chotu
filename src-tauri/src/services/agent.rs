use std::time::Duration;

use serde_json::{json, Value};

use crate::config::{ALL_MODELS_FAILED_MESSAGE, DEFAULT_AGENT_MODELS, MODEL_TIMEOUT_S};
use crate::services::{failover, llm::AiClient, tools::ToolRegistry};

const MAX_STEPS: usize = 6;
const AGENT_PROMPT: &str = "You are Chotu, an assistant on the user's Mac. \
Use a tool when it helps. Answer directly when it does not. Keep answers short.";

pub struct AgentRun {
    pub answer: String,
    pub steps: usize,
}

pub async fn agent_run(registry: &ToolRegistry, task: &str) -> Result<AgentRun, String> {
    let models = failover::model_list(
        std::env::var("AI_AGENT_MODELS").ok().as_deref(),
        &DEFAULT_AGENT_MODELS,
    );

    let tools = registry.definitions();
    let mut messages = vec![
        json!({ "role": "system", "content": AGENT_PROMPT }),
        json!({ "role": "user", "content": task }),
    ];

    let mut steps = 0;

    while steps < MAX_STEPS {
        let reply = ask_models(&models, &messages, &tools, true).await?;

        let calls = reply["tool_calls"].as_array().cloned().unwrap_or_default();
        if calls.is_empty() {
            return Ok(AgentRun {
                answer: text_of(&reply),
                steps,
            });
        }

        messages.push(json!({
            "role": "assistant",
            "content": reply["content"],
            "tool_calls": reply["tool_calls"],
        }));

        for call in &calls {
            let name = call["function"]["name"].as_str().unwrap_or("");

            let args = call["function"]["arguments"].as_str().unwrap_or("");
            let res = registry.run(name, args).await;

            messages.push(json!({
                "role": "tool",
                "tool_call_id": call["id"],
                "content": res.unwrap_or_else(|err| err),
            }));
            steps += 1;
        }
    }
    let reply = ask_models(&models, &messages, &tools, false).await?;
    let answer = text_of(&reply);
    if answer.is_empty() {
        return Err("The agent ran out of steps without an answer.".to_string());
    }
    Ok(AgentRun { answer, steps })
}

fn text_of(message: &Value) -> String {
    message["content"].as_str().unwrap_or("").trim().to_string()
}

async fn ask_models(
    models: &[String],
    messages: &[Value],
    tools: &[Value],
    allow_tools: bool,
) -> Result<Value, String> {
    let mut last_error = ALL_MODELS_FAILED_MESSAGE.to_string();
    for model in models {
        match ask_one_model(model, messages, tools, allow_tools).await {
            Ok(reply) => return Ok(reply),
            Err(error) => {
                log::warn!("agent model {model} failed: {error}");
                last_error = error;
            }
        }
    }
    Err(last_error)
}

async fn ask_one_model(
    model: &str,
    messages: &[Value],
    tools: &[Value],
    allow_tools: bool,
) -> Result<Value, String> {
    let ai = AiClient::shared();
    let body = json!({
        "model": model,
        "messages": messages,
        "tools": tools,
        "tool_choice": if allow_tools { "auto" } else { "none" },
        "max_tokens": 1500,
    });
    let response = reqwest::Client::new()
        .post(format!("{}/chat/completions", ai.url.trim_end_matches('/')))
        .bearer_auth(&ai.key)
        .timeout(Duration::from_secs(MODEL_TIMEOUT_S))
        .json(&body)
        .send()
        .await
        .map_err(|error| error.to_string())?;
    let body: Value = response.json().await.map_err(|error| error.to_string())?;
    usable_message(&body)
}

fn usable_message(body: &Value) -> Result<Value, String> {
    let message = &body["choices"][0]["message"];
    let has_calls = message["tool_calls"]
        .as_array()
        .is_some_and(|calls| !calls.is_empty());
    if text_of(message).is_empty() && !has_calls {
        let reason = body["error"]["message"].as_str();
        return Err(reason
            .unwrap_or("The model sent an empty reply.")
            .to_string());
    }
    Ok(message.clone())
}
