//! Web search: query DuckDuckGo, fetch the top pages, and format them as
//! numbered source blocks for the LLM prompt.
mod ddg;
mod page;

use std::sync::LazyLock;
use std::time::Duration;

use futures::future::join_all;

use crate::config::{BROWSER_USER_AGENT, PAGE_CONTENT_CHAR, TIMEOUT_S, TOP_PAGE_K};
use crate::models::web::SerpItem;

/// One shared HTTP client (connection pool) for all outbound requests.
fn http_client() -> &'static reqwest::Client {
    static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

    &CLIENT
}

async fn fetch_page(url: &str) -> Result<String, reqwest::Error> {
    let res = http_client()
        .get(url)
        .header("User-Agent", BROWSER_USER_AGENT)
        .send()
        .await?;

    res.text().await
}

async fn fetch_top_k(items: Vec<SerpItem>, limits: Option<usize>) -> Vec<SerpItem> {
    let top_k: usize = limits.unwrap_or(TOP_PAGE_K);
    let tasks = items.into_iter().take(top_k).map(fetch_and_extract);
    join_all(tasks).await
}

async fn fetch_and_extract(mut item: SerpItem) -> SerpItem {
    let fetched =
        tokio::time::timeout(Duration::from_secs(TIMEOUT_S), fetch_page(&item.source)).await;

    let extracted_data = match fetched {
        Ok(Ok(html)) => page::extract_page(&html, &item.source).ok(),
        _ => None,
    };

    if let Some(text) = extracted_data.filter(|t| !t.is_empty()) {
        item.content = text.chars().take(PAGE_CONTENT_CHAR).collect();
    }
    item
}

/// Numbered `[n] title (url)\n content` blocks, joined by blank lines.
fn build_serp_blocks(items: &[SerpItem]) -> String {
    items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            format!(
                "[{}] {} ({})\n {}",
                i + 1,
                item.title,
                item.source,
                item.content
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Ask DuckDuckGo and keep the top `TOP_PAGE_K` results (titles and links only).
/// An empty list is fine: the caller tells the AI that nothing was found.
pub async fn search_top(query: &str, lang: &str) -> Result<Vec<SerpItem>, String> {
    let mut items = ddg::search_ddg(query, false, lang)
        .await
        .map_err(|e| e.to_string())?;
    items.truncate(TOP_PAGE_K);
    Ok(items)
}

/// Download those pages and return (items, prompt-ready source blocks).
pub async fn read_pages(items: Vec<SerpItem>) -> (Vec<SerpItem>, String) {
    let fetched = fetch_top_k(items, None).await;
    let blocks = build_serp_blocks(&fetched);
    (fetched, blocks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_real_websearch() {
        let items = ddg::search_ddg(
            "is web3 is growing in 2026 after aug 2026 and what is growing?",
            false,
            "en",
        )
        .await
        .expect("search should succed!");

        let extracted_data = fetch_top_k(items, Some(4)).await;
        assert_eq!(extracted_data.len(), 4);
        for item in &extracted_data {
            println!("{} -> chars {}\n", item.source, item.content);
        }
    }
}
