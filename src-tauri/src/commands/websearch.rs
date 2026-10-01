use serde::Serialize;
use tauri::ipc::Channel;

use crate::adapters::websearch::web_search;
use crate::ai::client::AiClient;

#[derive(Serialize, Debug)]
pub struct WebSearchResponse {
    ans: String,
    sources: Vec<(String, String)>,
}

#[tauri::command]
pub async fn ask_web(channel: Channel<String>, query: String) -> Result<WebSearchResponse, String> {
    let (items, blocks) = web_search(&query, "en").await?;

    let ai_ans = AiClient::shared()
        .call_llm_for_web(&channel, &query, &blocks)
        .await?;
    let sources = items.into_iter().map(|i| (i.title, i.source)).collect();

    Ok(WebSearchResponse {
        ans: ai_ans,
        sources,
    })
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
        let (items, blocks) = web_search(query, "en")
            .await
            .expect("web_search should succeed");
        let ans = AiClient::shared()
            .call_llm_for_web(&channel, query, &blocks)
            .await
            .expect("call_llm_for_web should succeed");
        let sources = items.into_iter().map(|i| (i.title, i.source)).collect();
        let response = WebSearchResponse { ans, sources };

        assert!(!response.ans.is_empty());
        println!("{:#?}", response);
    }
}
