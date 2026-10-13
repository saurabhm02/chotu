use serde_json::Value;

use super::get_backup;
use crate::config::{WIKIPEDIA_EXTRACT_CHARS, WIKIPEDIA_RESULTS};
use crate::models::web::SerpItem;

/// "en" or "en-US" becomes "en". Anything that does not look like a language gives "en".
fn wikipedia_language(lang: &str) -> String {
    let code: String = lang.split('-').next().unwrap_or("").to_lowercase();
    let is_code = (2..=3).contains(&code.len()) && code.chars().all(|c| c.is_ascii_alphabetic());
    if is_code {
        code
    } else {
        "en".to_string()
    }
}

fn article_url(language: &str, title: &str) -> Option<String> {
    let mut url = url::Url::parse(&format!("https://{language}.wikipedia.org/wiki/")).ok()?;
    url.path_segments_mut()
        .ok()?
        .pop_if_empty()
        .push(&title.replace(' ', "_"));
    Some(url.to_string())
}

pub async fn search(query: &str, lang: &str) -> Vec<SerpItem> {
    let language = wikipedia_language(lang);
    let limit = WIKIPEDIA_RESULTS.to_string();
    let chars = WIKIPEDIA_EXTRACT_CHARS.to_string();
    let response = get_backup(
        "Wikipedia",
        &format!("https://{language}.wikipedia.org/w/api.php"),
        &[
            ("action", "query"),
            ("format", "json"),
            ("generator", "search"),
            ("gsrsearch", query),
            ("gsrlimit", &limit),
            ("prop", "extracts"),
            ("exintro", "1"),
            ("explaintext", "1"),
            ("exchars", &chars),
        ],
    )
    .await;

    let Some(response) = response else {
        return Vec::new();
    };
    match response.json::<Value>().await {
        Ok(body) => parse(&body, &language),
        Err(e) => {
            log::warn!("Wikipedia sent a reply we cannot read: {e}");
            Vec::new()
        }
    }
}

/// The articles of a search reply, best match first. Pages without text are skipped.
pub fn parse(body: &Value, language: &str) -> Vec<SerpItem> {
    let Some(pages) = body["query"]["pages"].as_object() else {
        return Vec::new();
    };

    let mut found: Vec<(i64, SerpItem)> = pages
        .values()
        .filter_map(|page| {
            let title = page["title"].as_str()?.trim();
            let extract = page["extract"].as_str()?.trim();
            if title.is_empty() || extract.is_empty() {
                return None;
            }
            let rank = page["index"].as_i64().unwrap_or(i64::MAX);
            let item = SerpItem {
                title: format!("{title} - Wikipedia"),
                source: article_url(language, title)?,
                content: extract.to_string(),
                needs_page: false,
            };
            Some((rank, item))
        })
        .collect();

    found.sort_by_key(|(rank, _)| *rank);
    found.into_iter().map(|(_, item)| item).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn real_reply() -> Value {
        json!({"batchcomplete":"","query":{"pages":{
            "81297551":{"pageid":81297551,"ns":0,"title":"Outline of the Rust programming language","index":2,
                "extract":"The following outline is provided as an overview of Rust."},
            "29414838":{"pageid":29414838,"ns":0,"title":"Rust (programming language)","index":1,
                "extract":"Rust is a general-purpose programming language that emphasizes performance."}
        }}})
    }

    #[test]
    fn articles_come_best_match_first_with_their_text() {
        let items = parse(&real_reply(), "en");
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "Rust (programming language) - Wikipedia");
        assert_eq!(
            items[0].source,
            "https://en.wikipedia.org/wiki/Rust_(programming_language)"
        );
        assert!(items[0].content.starts_with("Rust is a general-purpose"));
        assert!(!items[0].needs_page);
    }

    #[test]
    fn pages_without_text_and_odd_replies_give_nothing() {
        let empty_extract = json!({"query":{"pages":{"1":{"title":"X","extract":"  ","index":1}}}});
        assert!(parse(&empty_extract, "en").is_empty());
        assert!(parse(&json!({}), "en").is_empty());
        assert!(parse(&json!("text"), "en").is_empty());
    }

    #[test]
    fn the_article_link_uses_the_language_site() {
        let url = article_url("hi", "भारत").unwrap();
        assert!(url.starts_with("https://hi.wikipedia.org/wiki/"));
    }

    #[test]
    fn a_language_is_reduced_to_its_code() {
        assert_eq!(wikipedia_language("en"), "en");
        assert_eq!(wikipedia_language("en-US"), "en");
        assert_eq!(wikipedia_language("HI"), "hi");
        assert_eq!(wikipedia_language(""), "en");
        assert_eq!(wikipedia_language("../evil"), "en");
    }
}
