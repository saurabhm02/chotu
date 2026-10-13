//! Web search: query DuckDuckGo, fetch the top pages, and format them as
//! numbered source blocks for the LLM prompt.
mod ddg;
mod news;
mod page;
mod wikipedia;
mod weather;

use std::sync::LazyLock;
use std::time::Duration;

use futures::future::join_all;

use crate::config::{
    BACKUP_TIMEOUT_S, BACKUP_USER_AGENT, BROWSER_USER_AGENT, PAGE_CONTENT_CHAR, TIMEOUT_S,
    TOP_PAGE_K,
};
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
    if !item.needs_page {
        return item;
    }
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

const FRESH_WORDS: [&str; 16] = [
    "news",
    "latest",
    "today",
    "yesterday",
    "tonight",
    "now",
    "current",
    "currently",
    "price",
    "score",
    "won",
    "result",
    "results",
    "update",
    "breaking",
    "live",
];

fn is_fresh_question(question: &str) -> bool {
    question
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .any(|word| FRESH_WORDS.contains(&word))
}

/// News and Wikipedia results in one list, the likelier source first, without repeats.
fn combine(fresh: bool, news: Vec<SerpItem>, wikipedia: Vec<SerpItem>) -> Vec<SerpItem> {
    let (first, second) = if fresh {
        (news, wikipedia)
    } else {
        (wikipedia, news)
    };
    let mut items: Vec<SerpItem> = Vec::new();
    for item in first.into_iter().chain(second) {
        if !items.iter().any(|seen| seen.source == item.source) {
            items.push(item);
        }
    }
    items
}

/// A GET to a backup source. Any problem (offline, slow, an error status) gives `None`.
async fn get_backup(source: &str, url: &str, query: &[(&str, &str)]) -> Option<reqwest::Response> {
    let response = http_client()
        .get(url)
        .query(query)
        .header("User-Agent", BACKUP_USER_AGENT)
        .timeout(Duration::from_secs(BACKUP_TIMEOUT_S))
        .send()
        .await
        .inspect_err(|e| log::warn!("{source} did not answer: {e}"))
        .ok()?;

    if !response.status().is_success() {
        log::warn!("{source} answered with {}", response.status());
        return None;
    }
    Some(response)
}

/// Used when DuckDuckGo gives nothing (blocked, offline, or no results). A weather question
/// goes to Open-Meteo. Anything else goes to Google News and Wikipedia.
async fn search_backup(query: &str, lang: &str) -> Vec<SerpItem> {
    if weather::is_weather_question(query) {
        let forecast = weather::search(query, lang).await;
        if !forecast.is_empty() {
            return forecast;
        }
    }

    let (news, wikipedia) =
        futures::join!(news::search(query, lang), wikipedia::search(query, lang));
    combine(is_fresh_question(query), news, wikipedia)
}

/// Ask DuckDuckGo and keep the top `TOP_PAGE_K` results. When DuckDuckGo gives nothing, the
/// backup sources are tried. An empty list is fine: the caller tells the AI that nothing was found.
pub async fn search_top(query: &str, lang: &str) -> Result<Vec<SerpItem>, String> {
    let mut items = ddg::search_ddg(query, false, lang)
        .await
        .inspect_err(|e| log::warn!("DuckDuckGo did not answer: {e}"))
        .unwrap_or_default();

    if items.is_empty() {
        log::warn!("DuckDuckGo gave nothing for {query:?}, trying the backup sources");
        items = search_backup(query, lang).await;
    }

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

    fn item(source: &str) -> SerpItem {
        SerpItem {
            title: source.to_string(),
            source: source.to_string(),
            content: String::new(),
            needs_page: false,
        }
    }

    #[test]
    fn fresh_words_are_found_whatever_the_case_or_punctuation() {
        assert!(is_fresh_question("Who WON the match yesterday?"));
        assert!(is_fresh_question("bitcoin price, now"));
        assert!(!is_fresh_question("what is a closure in rust"));
    }

    #[test]
    fn news_comes_first_for_fresh_questions_and_wikipedia_otherwise() {
        let news = vec![item("news-1")];
        let wiki = vec![item("wiki-1")];
        let fresh: Vec<String> = combine(true, news.clone(), wiki.clone())
            .into_iter()
            .map(|i| i.source)
            .collect();
        assert_eq!(fresh, vec!["news-1", "wiki-1"]);
        let stable: Vec<String> = combine(false, news, wiki)
            .into_iter()
            .map(|i| i.source)
            .collect();
        assert_eq!(stable, vec!["wiki-1", "news-1"]);
    }

    #[test]
    fn the_same_link_is_kept_once() {
        let merged = combine(true, vec![item("a")], vec![item("a"), item("b")]);
        assert_eq!(merged.len(), 2);
    }

    #[tokio::test]
    #[ignore]
    async fn backup_sources_for_real() {
        dotenvy::dotenv().ok();
        let questions = [
            "what is the weather in Pune tomorrow",
            "who won the india australia match yesterday",
            "what is the rust programming language",
        ];
        for question in questions {
            let items = search_backup(question, "en").await;
            println!("\n[{question}] -> {} items", items.len());
            for item in items.iter().take(3) {
                let text: String = item.content.chars().take(110).collect();
                println!("   - {}\n     {}\n     {text}", item.title, item.source);
            }
        }
    }

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
