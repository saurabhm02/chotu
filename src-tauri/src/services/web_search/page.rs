//! Pull the readable text out of a fetched web page.
use readability::extractor;
use std::io::Cursor;
use url::Url;

pub fn extract_page(html: &str, page_url: &str) -> Result<String, String> {
    let parse_url = Url::parse(page_url).map_err(|e| e.to_string())?;
    let mut cursor = Cursor::new(html.as_bytes());

    let item = extractor::extract(&mut cursor, &parse_url).map_err(|e| e.to_string())?;
    Ok(item.text.trim().to_string())
}
