use serde::Serialize;

use crate::adapters::websearch::web_search;
use crate::ai::client::AiClient;

#[derive(Serialize, Debug)]
pub struct WebSearchResponse {
    ans: String,
    sources: Vec<(String, String)>,
}

#[tauri::command]
pub async fn ask_web(query: String) -> Result<WebSearchResponse, String> {
    let (items, blocks) = web_search(&query, "en").await?;

    let ai_ans = AiClient::new().call_llm_for_web(&query, &blocks).await?;
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

        let response =
            ask_web("is web3 is growing in 2026 after aug 2026 and what is growing?".to_string())
                .await
                .expect("ask_web should succeed");

        assert!(!response.ans.is_empty());
        println!("{:#?}", response);
    }
}
