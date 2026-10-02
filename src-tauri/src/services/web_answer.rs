//! "/web" end to end: search the web, then have the LLM answer from the results.
use tauri::ipc::Channel;

use super::llm::AiClient;
use super::web_search::web_search;
use crate::models::web::WebSearchResponse;

pub async fn answer(channel: &Channel<String>, query: &str) -> Result<WebSearchResponse, String> {
    let (items, blocks) = web_search(query, "en").await?;

    let ans = AiClient::shared()
        .call_llm_for_web(channel, query, &blocks)
        .await?;
    let sources = items.into_iter().map(|i| (i.title, i.source)).collect();

    Ok(WebSearchResponse { ans, sources })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[tokio::test]
    async fn test_ask_web() {
        dotenvy::dotenv().ok();

        let channel: Channel<String> = Channel::new(|_msg| Ok(()));
        let query = "is web3 is growing in 2026 after aug 2026 and what is growing?";
        let response = answer(&channel, query).await.expect("answer should succeed");

        assert!(!response.ans.is_empty());
        println!("{:#?}", response);
    }
}
