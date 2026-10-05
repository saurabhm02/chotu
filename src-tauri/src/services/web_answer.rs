//! "/web" end to end: search the web, then have the LLM answer from the results.
use tauri::ipc::Channel;

use super::llm::AiClient;
use super::web_search::{read_pages, search_top};
use crate::config::SEARCH_QUOTE_CHARS;
use crate::models::chat::ChatMessage;
use crate::models::stream::{Phase, Source, StreamEvent};
use crate::models::web::WebSearchResponse;

/// The words we send to the search engine: the question, then the start of the
/// highlighted text (if any) squeezed onto one line.
fn search_text(query: &str, context: Option<&str>) -> String {
    let quote = context
        .unwrap_or("")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let quote: String = quote.chars().take(SEARCH_QUOTE_CHARS).collect();

    format!("{} {}", query.trim(), quote).trim().to_string()
}

pub async fn answer(
    channel: &Channel<StreamEvent>,
    query: &str,
    context: Option<&str>,
    history: &[ChatMessage],
) -> Result<WebSearchResponse, String> {
    let _ = channel.send(StreamEvent::Status(Phase::Searching));
    let found = search_top(&search_text(query, context), "en").await?;
    if found.is_empty() {
        log::warn!("web search found nothing for {query:?}, asking the AI without sources");
    }

    // Tell the screen which sites we are about to read, before the slow part.
    let list = found
        .iter()
        .map(|i| Source {
            title: i.title.clone(),
            url: i.source.clone(),
        })
        .collect();
    if !found.is_empty() {
        let _ = channel.send(StreamEvent::Sources(list));
        let _ = channel.send(StreamEvent::Status(Phase::Reading));
    }

    let (items, blocks) = read_pages(found).await;

    // Pages are read. From here we only wait for the model.
    let _ = channel.send(StreamEvent::Status(Phase::Thinking));

    let ans = AiClient::shared()
        .call_llm_for_web(channel, query, context, history, &blocks)
        .await?;
    let sources = items.into_iter().map(|i| (i.title, i.source)).collect();

    Ok(WebSearchResponse { ans, sources })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_is_just_the_question_without_a_quote() {
        assert_eq!(search_text(" what is rust? ", None), "what is rust?");
        assert_eq!(search_text("what is rust?", Some("  \n ")), "what is rust?");
    }

    #[test]
    fn quote_follows_the_question_on_one_line() {
        assert_eq!(
            search_text("is this true?", Some("beads are\n  seeds")),
            "is this true? beads are seeds"
        );
    }

    #[test]
    fn only_the_start_of_a_long_quote_is_used() {
        let long = "a".repeat(500);
        let text = search_text("q", Some(&long));
        assert_eq!(text.chars().count(), 2 + SEARCH_QUOTE_CHARS);
    }

    #[test]
    fn quote_alone_is_searched_when_there_is_no_question() {
        assert_eq!(search_text("", Some("rudraksha")), "rudraksha");
    }

    #[ignore]
    #[tokio::test]
    async fn test_ask_web() {
        dotenvy::dotenv().ok();

        let channel: Channel<StreamEvent> = Channel::new(|_msg| Ok(()));
        let query = "is web3 is growing in 2026 after aug 2026 and what is growing?";
        let response = answer(&channel, query, None, &[])
            .await
            .expect("answer should succeed");

        assert!(!response.ans.is_empty());
        println!("{:#?}", response);
    }
}
