use serde::Serialize;

/// One search-engine result row, later filled with the page text.
#[derive(Clone, Debug)]
pub struct SerpItem {
    pub title: String,
    pub source: String,
    pub content: String,
}

/// What the web command returns to the frontend: the answer plus (title, url) sources.
#[derive(Serialize, Debug)]
pub struct WebSearchResponse {
    pub ans: String,
    pub sources: Vec<(String, String)>,
}
