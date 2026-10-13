use std::time::Duration;

use crate::config::{
    DECIDE_HINT_MESSAGE, DECISION_MODELS_UNAVAILABLE_MESSAGE, DECISION_SLOW_TIMEOUT_S,
};
use crate::services::decide::DecisionClient;
use crate::services::llm::AiClient;
use crate::utils::decide::{build_question, format_choice, parse_options_reply, QUESTION_ID};

#[tauri::command]
pub async fn decide_between(text: Option<String>, question: String) -> Result<String, String> {
    let question = question.trim();
    if question.is_empty() {
        return Ok(DECIDE_HINT_MESSAGE.to_string());
    }

    let reply = match AiClient::shared().list_options(question).await {
        Ok(reply) => reply,
        Err(e) => {
            log::warn!("could not read the options: {e}");
            return Ok(DECISION_MODELS_UNAVAILABLE_MESSAGE.to_string());
        }
    };
    let options = match parse_options_reply(&reply, question) {
        Ok(options) => options,
        Err(hint) => return Ok(hint),
    };

    let context = text
        .as_deref()
        .map(str::trim)
        .filter(|text| !text.is_empty());
    let state = match context {
        Some(context) => format!("{question}\n\n{context}"),
        None => question.to_string(),
    };

    let answers = DecisionClient::shared()
        .ask_questions(
            &state,
            &[build_question(&options)],
            Duration::from_secs(DECISION_SLOW_TIMEOUT_S),
        )
        .await;

    let choice = answers
        .as_ref()
        .and_then(|answers| answers.choice(QUESTION_ID));
    Ok(format_choice(&options, choice))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn an_empty_question_answers_with_a_hint_and_does_not_call_the_ai() {
        let reply = decide_between(None, "   ".to_string()).await;
        assert_eq!(reply, Ok(DECIDE_HINT_MESSAGE.to_string()));
    }

    #[tokio::test]
    #[ignore]
    async fn decide_for_real() {
        dotenvy::dotenv().ok();
        let cases = [
            ("what should i eat pizza or burger", None),
            ("what should i eat", None),
            ("is rust better than go?", None),
            ("what should i have for dessert ice cream or cake?", None),
            ("should i go to the gym or stay home?", None),
            (
                "tea, coffee or juice",
                Some("I could not sleep last night and have a meeting at 9"),
            ),
            (
                "which is bigger an elephant or a mouse or a cat or an ant",
                None,
            ),
            ("pizza", None),
        ];
        for (question, context) in cases {
            let started = std::time::Instant::now();
            let reply = decide_between(context.map(str::to_string), question.to_string())
                .await
                .unwrap();
            println!(
                "\n[{question}] ({} ms)\n{reply}",
                started.elapsed().as_millis()
            );
        }
    }
}
