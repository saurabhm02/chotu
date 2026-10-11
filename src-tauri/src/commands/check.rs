use crate::services::decide::DecisionClient;
use crate::utils::check::{format_response, parse_check_input};

#[tauri::command]
pub async fn check_text(text: Option<String>, question: String) -> Result<String, String> {
    let input = match parse_check_input(text.as_deref(), &question) {
        Ok(input) => input,
        Err(hint) => return Ok(hint.to_string()),
    };

    let decision = DecisionClient::shared()
        .ask_yes_or_no(input.text, input.question)
        .await;
    Ok(format_response(decision))
}

#[cfg(test)]
mod tests {
    use crate::config::CHECK_MISSING_QUESTION_MESSAGE;

    use super::*;

    #[tokio::test]
    async fn check_question_answer_with_hint_and_should_not_call_ai() {
        let reply = check_text(Some("text".to_string()), "".to_string()).await;
        assert_eq!(reply, Ok(CHECK_MISSING_QUESTION_MESSAGE.to_string()))
    }

    #[tokio::test]
    #[ignore]
    async fn check_answers_for_real() {
        dotenvy::dotenv().ok();
        let cases = [
            (
                "Congratulations! You won $1,000,000. Send your bank details to claim.",
                "is this a scam?",
            ),
            (
                "Hi, lunch tomorrow? I will bring the notes.",
                "is this a scam?",
            ),
            (
                "Send me the report by Friday 5pm or we lose the client.",
                "does it mention a deadline?",
            ),
            (
                "Thanks a lot for your help, really appreciate it!",
                "is this message rude?",
            ),
        ];
        for (text, question) in cases {
            let reply = check_text(Some(text.to_string()), question.to_string())
                .await
                .unwrap();
            println!("\n[{question}] {text}\n{reply}");
        }
    }
}
