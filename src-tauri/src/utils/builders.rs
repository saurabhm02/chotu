//! Small, pure, endpoint-agnostic helpers for building outbound HTTP requests.
//! Anything here must not be tied to one specific API's field names or shape —
//! that kind of logic belongs next to the request builder that owns it
//! (e.g. `adapters::websearch::ddg_form_data`).

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

use crate::types::constants::BROWSER_USER_AGENT;

use super::extractor::SerpItem;

/// Builds a header set for a scrape-style request: a real browser User-Agent
/// (so the target site doesn't reject an obvious bot), an HTML `Accept`,
/// the caller's `Accept-Language`, and a `Referer`. Reusable for any outbound
/// fetch that needs to look like a real browser, not just one endpoint.
pub fn build_scrape_headers(lang: &str, referer: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();

    let mut add_header = |key: &'static str, value: &str| {
        if let (Ok(key), Ok(value)) = (HeaderName::try_from(key), HeaderValue::try_from(value)) {
            headers.insert(key, value);
        }
    };

    add_header("User-Agent", BROWSER_USER_AGENT);
    add_header("Accept", "text/html,application/xhtml+xml");
    add_header("Accept-Language", lang);
    add_header("Referer", referer);

    headers
}

pub fn build_serp_blocks(items: &[SerpItem]) -> String {
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
