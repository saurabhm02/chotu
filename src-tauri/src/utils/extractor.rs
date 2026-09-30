use scraper::{Html, Selector};

#[derive(Clone, Debug)]
pub struct SerpItem {
    pub title: String,
    pub source: String,
    pub content: String,
}

pub mod ddg {
    use super::{Html, Selector, SerpItem};

    pub fn parse_html(body: &str) -> Vec<SerpItem> {
        let row_selector = Selector::parse("div.result").expect("Valid Selector");
        let title_selector = Selector::parse("a.result__a").expect("Valid Selector");
        let snippet_selector = Selector::parse("a.result__snippet").expect("Valid Selector");

        let document = Html::parse_document(body);
        let mut items = Vec::new();

        for row in document.select(&row_selector) {
            let Some(anchor) = row.select(&title_selector).next() else {
                continue;
            };

            let title = anchor.text().collect::<String>().trim().to_string();

            let Some(href) = anchor.value().attr("href") else {
                continue;
            };
            let url = parse_url(href);

            if title.is_empty() || url.is_empty() {
                continue;
            }

            if is_ad(row.value().attr("class"), &url) {
                continue;
            }

            let desc = row
                .select(&snippet_selector)
                .next()
                .map(|s| s.text().collect::<String>().trim().to_string())
                .unwrap_or_default();

            items.push(SerpItem {
                title,
                source: url,
                content: desc,
            });
        }

        items
    }

    fn parse_url(href: &str) -> String {
        let path = if href.starts_with("https://") || href.starts_with("http://") {
            href.to_string()
        } else if let Some(rest) = href.strip_prefix("//") {
            format!("https://{rest}")
        } else if let Some(rest) = href.strip_prefix('/') {
            format!("https://duckduckgo.com/{rest}")
        } else {
            return href.to_string();
        };

        match path.find("uddg=") {
            Some(idx) => {
                let after = &path[idx + "uddg=".len()..];
                let raw = after.split('&').next().unwrap_or(after);
                percent_decode(raw)
            }
            None => path,
        }
    }

    /// Minimal percent-decoder for `%XX` escape sequences (e.g. `%3A` -> `:`).
    /// Non-UTF8 or malformed sequences fall back to the original characters
    /// rather than panicking.
    fn percent_decode(s: &str) -> String {
        let bytes = s.as_bytes();
        let mut out = Vec::with_capacity(bytes.len());
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'%' && i + 2 < bytes.len() {
                let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or("");
                if let Ok(byte) = u8::from_str_radix(hex, 16) {
                    out.push(byte);
                    i += 3;
                    continue;
                }
            }
            out.push(bytes[i]);
            i += 1;
        }
        String::from_utf8_lossy(&out).into_owned()
    }

    /// ddg marks sponsored rows with a `result--ad` class, or points them at the
    /// `y.js` ad-redirect endpoint instead of `l/`.
    fn is_ad(class_attr: Option<&str>, source: &str) -> bool {
        class_attr.is_some_and(|c| c.contains("result--ad"))
            || source.contains("duckduckgo.com/y.js")
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        const SAMPLE_HTML: &str = r#"
               <div class="result results_links results_links_deep web-result ">
                 <div class="links_main links_deep result__body">
                   <h2 class="result__title">
                     <a rel="nofollow" class="result__a" href="https://medium.com/rrf-explained">
                       Reciprocal Rank Fusion (RRF) explained in 4 mins. - Medium
                     </a>
                   </h2>
                   <a class="result__snippet" href="https://medium.com/rrf-explained">
                     Unlock the power of Reciprocal Rank Fusion (<b>RRF</b>) to supercharge search.
                   </a>
                 </div>
               </div>
               <div class="result results_links web-result ">
                 <div class="links_main result__body">
                   <h2 class="result__title">
                     <a rel="nofollow" class="result__a" href="https://learn.microsoft.com/rrf">
                       Hybrid Search Scoring (RRF) - Azure AI Search
                     </a>
                   </h2>
                   <a class="result__snippet" href="https://learn.microsoft.com/rrf">
                     Reciprocal Rank Fusion is an algorithm that merges search scores.
                   </a>
                 </div>
               </div>
           "#;

        #[test]
        fn parse_sample_results_with_title_descriptin() {
            let items = parse_html(SAMPLE_HTML);
            assert_eq!(items.len(), 2);

            assert_eq!(
                items[0].title,
                "Reciprocal Rank Fusion (RRF) explained in 4 mins. - Medium"
            );

            assert_eq!(items[0].source, "https://medium.com/rrf-explained");
            assert_eq!(items[1].source, "https://learn.microsoft.com/rrf");
            assert!(items[0].content.contains("RRF"));
        }

        #[test]
        fn test_empty_state() {
            let items = parse_html("<html><body>nothing here</body></html>");

            assert!(items.is_empty());
        }

        #[test]
        fn should_skip_the_ad_result() {
            let body = r#"
                   <div class="result result--ad">
                     <h2 class="result__title">
                       <a class="result__a" href="https://duckduckgo.com/y.js?ad_domain=spam.example">Sponsored</a>
                     </h2>
                   </div>
                   <div class="result">
                     <h2 class="result__title">
                       <a class="result__a" href="https://real.example/">Real result</a>
                     </h2>
                   </div>
               "#;

            let items = parse_html(body);

            assert_eq!(items.len(), 1);
            assert_eq!(items[0].source, "https://real.example/");
        }
    }
}

pub mod article {

    use readability::extractor;
    use std::io::Cursor;
    use url::Url;

    pub fn extract_page(html: &str, page_url: &str) -> Result<String, String> {
        let parse_url = Url::parse(page_url).map_err(|e| e.to_string())?;
        let mut cursor = Cursor::new(html.as_bytes());

        let item = extractor::extract(&mut cursor, &parse_url).map_err(|e| e.to_string())?;
        Ok(item.text.trim().to_string())
    }
}
