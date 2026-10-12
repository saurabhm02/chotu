use std::time::Duration;

use tauri::ipc::Channel;

use crate::config::{AUTO_WEB_QUESTION, AUTO_WEB_TIMEOUT_S};
use crate::models::chat::ChatMessage;
use crate::models::stream::StreamEvent;
use crate::models::web::WebSearchResponse;
use crate::services::decide::DecisionClient;
use crate::services::web::{self, auto_web_enabled, need_web_search};
#[tauri::command]
pub async fn ask_web(
    channel: Channel<StreamEvent>,
    query: String,
    context: Option<String>,
    history: Option<Vec<ChatMessage>>,
) -> Result<WebSearchResponse, String> {
    web::answer(
        &channel,
        &query,
        context.as_deref(),
        &history.unwrap_or_default(),
    )
    .await
}

#[tauri::command]
pub async fn should_search_web(question: String) -> Result<bool, String> {
    let search = need_web_search(auto_web_enabled(), &question, |text| async move {
        let ask = DecisionClient::shared().ask_yes_or_no(&text, AUTO_WEB_QUESTION);
        tokio::time::timeout(Duration::from_secs(AUTO_WEB_TIMEOUT_S), ask)
            .await
            .ok()
            .flatten()
    })
    .await;

    log::info!("auto web search decided: {search}");
    Ok(search)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn auto_web_for_real() {
        dotenvy::dotenv().ok();
        let questions = [
            "what is the bitcoin price right now?",
            "who won the cricket match yesterday?",
            "what is the latest version of rust?",
            "weather in Pune tomorrow",
            "news about openai today",
            "is web3 growing in 2026?",
            "what is a closure in rust?",
            "write a haiku about rain",
            "explain recursion in simple words",
            "what is the capital of France?",
            "how do I reverse a string in python?",
            "why is the sky blue?",
        ];
        for question in questions {
            let search = should_search_web(question.to_string()).await.unwrap();
            println!("{search:5}  {question}");
        }
    }
}
