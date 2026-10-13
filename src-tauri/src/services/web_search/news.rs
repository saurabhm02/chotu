use std::env;
use std::sync::OnceLock;

use super::get_backup;
use crate::config::{DEFAULT_NEWS_REGION, NEWS_RESULTS};
use crate::models::web::SerpItem;

static REGION: OnceLock<String> = OnceLock::new();

/// A two letter country code in capitals. A missing or odd setting gives the default.
fn region_from(setting: Option<&str>) -> String {
    let wanted = setting.unwrap_or("").trim().to_uppercase();
    let is_country = wanted.len() == 2 && wanted.chars().all(|c| c.is_ascii_alphabetic());
    if is_country {
        wanted
    } else {
        DEFAULT_NEWS_REGION.to_string()
    }
}

/// The country from `NEWS_REGION` in `.env`, read once.
fn region() -> &'static str {
    REGION.get_or_init(|| {
        let region = region_from(env::var("NEWS_REGION").ok().as_deref());
        log::info!("news region: {region}");
        region
    })
}

pub async fn search(query: &str, lang: &str) -> Vec<SerpItem> {
    let language = lang.split('-').next().unwrap_or("en");
    let region = region();
    let hl = format!("{language}-{region}");
    let ceid = format!("{region}:{language}");
    let response = get_backup(
        "Google News",
        "https://news.google.com/rss/search",
        &[
            ("q", query),
            ("hl", &hl),
            ("gl", region),
            ("ceid", &ceid),
        ],
    )
    .await;

    let Some(response) = response else {
        return Vec::new();
    };
    match response.text().await {
        Ok(xml) => parse_rss(&xml),
        Err(e) => {
            log::warn!("Google News sent a reply we cannot read: {e}");
            Vec::new()
        }
    }
}

/// The text between `<tag ...>` and `</tag>`, with the XML escapes undone.
fn tag_text(block: &str, tag: &str) -> Option<String> {
    let from_open = &block[block.find(&format!("<{tag}"))?..];
    let inner = &from_open[from_open.find('>')? + 1..];
    let end = inner.find(&format!("</{tag}>"))?;
    Some(decode_entities(inner[..end].trim()))
}

fn decode_entities(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

/// The first `NEWS_RESULTS` headlines of an RSS feed. The headline already ends with the
/// publisher ("... - NDTV"), and the publish time is added to the text.
pub fn parse_rss(xml: &str) -> Vec<SerpItem> {
    xml.split("<item>")
        .skip(1)
        .filter_map(|chunk| {
            let item = chunk.split("</item>").next()?;
            let title = tag_text(item, "title")?;
            let link = tag_text(item, "link")?;
            if title.is_empty() || !link.starts_with("http") {
                return None;
            }
            let content = match tag_text(item, "pubDate") {
                Some(published) if !published.is_empty() => {
                    format!("{title} (published {published})")
                }
                _ => title.clone(),
            };
            Some(SerpItem {
                title,
                source: link,
                content,
                needs_page: false,
            })
        })
        .take(NEWS_RESULTS)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const FEED: &str = r#"<?xml version="1.0"?><rss version="2.0"><channel>
<title>"iphone price" - Google News</title><link>https://news.google.com/search?q=x</link>
<item><title>iPhone 18 Pro price in India &amp; US: what&#39;s new - Esquire India</title>
<link>https://news.google.com/rss/articles/CBMi1?oc=5</link><guid isPermaLink="false">CBMi1</guid>
<pubDate>Tue, 13 Oct 2026 08:00:00 GMT</pubDate><source url="https://www.esquireindia.com">Esquire India</source></item>
<item><title>Costly iPhone faces a reality check - NDTV</title>
<link>https://news.google.com/rss/articles/CBMi2?oc=5</link>
<pubDate>Mon, 12 Oct 2026 10:30:00 GMT</pubDate></item>
<item><title>No link here</title></item>
</channel></rss>"#;

    #[test]
    fn the_region_is_a_two_letter_country_in_capitals() {
        assert_eq!(region_from(Some("us")), "US");
        assert_eq!(region_from(Some(" gb ")), "GB");
        assert_eq!(region_from(Some("DE")), "DE");
    }

    #[test]
    fn a_missing_or_odd_region_gives_the_default() {
        for setting in [None, Some(""), Some("india"), Some("U5"), Some("../")] {
            assert_eq!(region_from(setting), DEFAULT_NEWS_REGION, "{setting:?}");
        }
    }

    #[test]
    fn headlines_are_read_with_their_time() {
        let items = parse_rss(FEED);
        assert_eq!(items.len(), 2);
        assert_eq!(
            items[0].title,
            "iPhone 18 Pro price in India & US: what's new - Esquire India"
        );
        assert_eq!(
            items[0].source,
            "https://news.google.com/rss/articles/CBMi1?oc=5"
        );
        assert!(items[0].content.contains("published Tue, 13 Oct 2026"));
        assert!(!items[0].needs_page);
    }

    #[test]
    fn the_feed_title_and_items_without_a_link_are_skipped() {
        let titles: Vec<String> = parse_rss(FEED).into_iter().map(|i| i.title).collect();
        assert!(!titles.iter().any(|t| t.contains("Google News")));
        assert!(!titles.iter().any(|t| t.contains("No link here")));
    }

    #[test]
    fn junk_gives_nothing() {
        assert!(parse_rss("").is_empty());
        assert!(parse_rss("<html>blocked</html>").is_empty());
    }

    #[test]
    fn at_most_the_limit_is_kept() {
        let item = "<item><title>T</title><link>https://example.com/a</link></item>";
        let feed = item.repeat(NEWS_RESULTS + 3);
        assert_eq!(parse_rss(&feed).len(), NEWS_RESULTS);
    }
}
