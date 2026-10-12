//! "/web" end to end: search the web, then have the LLM answer from the results.
use std::env;
use std::future::Future;
use std::sync::OnceLock;
use tauri::ipc::Channel;

use super::llm::AiClient;
use super::web_search::{read_pages, search_top};
use crate::config::SEARCH_SELECTED_CHARS;
use crate::models::chat::ChatMessage;
use crate::models::decision::{Decision, Verdict};
use crate::models::stream::{Phase, Source, StreamEvent};
use crate::models::web::WebSearchResponse;
use crate::utils::web::{is_auto_web_enabled, is_worth_asking};

static ENABLED: OnceLock<bool> = OnceLock::new();

/// The words we send to the search engine: the question, then the start of the
/// highlighted text (if any) squeezed onto one line.
fn search_text(query: &str, context: Option<&str>) -> String {
    let selected = context
        .unwrap_or("")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let selected: String = selected.chars().take(SEARCH_SELECTED_CHARS).collect();

    format!("{} {}", query.trim(), selected).trim().to_string()
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

pub fn auto_web_enabled() -> bool {
    *ENABLED.get_or_init(|| {
        let enabled = is_auto_web_enabled(env::var("AUTO_WEB").ok().as_deref());
        log::info!("auto web search: enabled:{enabled}");
        enabled
    })
}

pub async fn need_web_search<F, Fut>(enabled: bool, question: &str, ask_model: F) -> bool
where
    F: FnOnce(String) -> Fut,
    Fut: Future<Output = Option<Decision>>,
{
    if !enabled || !is_worth_asking(question) {
        return false;
    }
    matches!(
        ask_model(question.trim().to_string()).await,
        Some(Decision {
            verdict: Verdict::Yes,
            ..
        })
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_is_just_the_question_without_selected_text() {
        assert_eq!(search_text(" what is rust? ", None), "what is rust?");
        assert_eq!(search_text("what is rust?", Some("  \n ")), "what is rust?");
    }

    #[test]
    fn selected_text_follows_the_question_on_one_line() {
        assert_eq!(
            search_text("is this true?", Some("beads are\n  seeds")),
            "is this true? beads are seeds"
        );
    }

    #[test]
    fn only_the_start_of_long_selected_text_is_used() {
        let long = "a".repeat(500);
        let text = search_text("q", Some(&long));
        assert_eq!(text.chars().count(), 2 + SEARCH_SELECTED_CHARS);
    }

    #[test]
    fn selected_text_alone_is_searched_when_there_is_no_question() {
        assert_eq!(search_text("", Some("rudraksha")), "rudraksha");
    }

    async fn run(enabled: bool, question: &str, model_says: Option<f32>) -> (bool, u32) {
        let calls = std::cell::Cell::new(0);
        let search = need_web_search(enabled, question, |_text| {
            calls.set(calls.get() + 1);
            async move { model_says.map(Decision::from_probability) }
        })
        .await;
        (search, calls.get())
    }

    #[tokio::test]
    async fn a_sure_yes_searches() {
        assert_eq!(
            run(true, "bitcoin price right now", Some(0.95)).await,
            (true, 1)
        );
    }

    #[tokio::test]
    async fn a_no_an_unsure_answer_or_no_answer_does_not_search() {
        for answer in [Some(0.05), Some(0.55), None] {
            assert_eq!(
                run(true, "what is a closure in rust", answer).await,
                (false, 1),
                "{answer:?}"
            );
        }
    }

    #[tokio::test]
    async fn switched_off_never_asks_the_model() {
        assert_eq!(
            run(false, "bitcoin price right now", Some(0.99)).await,
            (false, 0)
        );
    }

    #[tokio::test]
    async fn a_one_word_question_never_asks_the_model() {
        assert_eq!(run(true, "thanks", Some(0.99)).await, (false, 0));
    }

    #[tokio::test]
    async fn the_model_gets_the_trimmed_question() {
        let search = need_web_search(true, "  weather today \n", |text| async move {
            assert_eq!(text, "weather today");
            Some(Decision::from_probability(0.9))
        })
        .await;
        assert!(search);
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
