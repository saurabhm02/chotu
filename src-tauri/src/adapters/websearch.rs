use std::time::Duration;

use futures::future::join_all;
use reqwest::header::HeaderValue;

use crate::types::constants::{
    BROWSER_USER_AGENT, DDG_URL, PAGE_CONTENT_CHAR, TIMEOUT_S, TOP_PAGE_K,
};
use crate::utils::{
    builders::{build_scrape_headers, build_serp_blocks},
    extractor::{article, ddg, SerpItem},
    statics::http_client,
};

/// DuckDuckGo's own `/html` form field names (`q`, `kl`) — specific to this
/// one endpoint, not a general-purpose form builder, so it lives here next to
/// the only request builder that uses it rather than in a generic utils file.
fn ddg_form_data(query: &str, lang: &str) -> Vec<(String, String)> {
    vec![
        ("q".to_string(), query.to_string()),
        ("kl".to_string(), lang.to_string()),
    ]
}

#[allow(dead_code)]
pub async fn ddg_request(
    query: &str,
    need_recent: bool,
    lang: &str,
) -> Result<String, reqwest::Error> {
    let client = http_client();
    let mut form = ddg_form_data(query, lang);
    let mut headers = build_scrape_headers(lang, DDG_URL);

    if need_recent {
        form.push(("df".to_string(), "w".to_string()));
        headers.insert(reqwest::header::COOKIE, HeaderValue::from_static("df=w"));
    }

    let res = client
        .post(DDG_URL.to_string())
        .headers(headers)
        .form(&form)
        .send()
        .await?;

    res.text().await
}

pub async fn search_ddg(
    query: &str,
    need_recent: bool,
    lang: &str,
) -> Result<Vec<SerpItem>, reqwest::Error> {
    let body = ddg_request(query, need_recent, lang).await?;

    Ok(ddg::parse_html(&body))
}

#[allow(dead_code)]
pub async fn fetch_page(url: &str) -> Result<String, reqwest::Error> {
    let res = http_client()
        .get(url)
        .header("User-Agent", BROWSER_USER_AGENT)
        .send()
        .await?;

    res.text().await
}

#[allow(dead_code)]
async fn fetch_top_k(items: Vec<SerpItem>, limits: Option<usize>) -> Vec<SerpItem> {
    let top_k: usize = limits.unwrap_or(TOP_PAGE_K);
    let tasks = items.into_iter().take(top_k).map(fetch_and_extract);
    join_all(tasks).await
}

#[allow(dead_code)]
async fn fetch_and_extract(mut item: SerpItem) -> SerpItem {
    let fetched =
        tokio::time::timeout(Duration::from_secs(TIMEOUT_S), fetch_page(&item.source)).await;

    let extracted_data = match fetched {
        Ok(Ok(html)) => article::extract_page(&html, &item.source).ok(),
        _ => None,
    };

    if let Some(text) = extracted_data.filter(|t| !t.is_empty()) {
        item.content = text.chars().take(PAGE_CONTENT_CHAR).collect();
    }
    item
}

pub async fn web_search(query: &str, lang: &str) -> Result<(Vec<SerpItem>, String), String> {
    let items = search_ddg(query, false, lang)
        .await
        .map_err(|e| e.to_string())?;
    let fetched = fetch_top_k(items, None).await;
    let blocks = build_serp_blocks(&fetched);

    Ok((fetched, blocks))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_real_websearch() {
        let items = search_ddg(
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
