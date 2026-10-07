//! Rebuilds on-screen chat turns from saved messages.

use crate::models::chat::StoredMessage;
use crate::models::ChatTurn;

/// A "user" message starts a turn and the "assistant" message after it answers it.
/// `clock` formats a saved timestamp for display.
pub fn turns_from_messages(
    messages: &[StoredMessage],
    clock: impl Fn(i64) -> String,
) -> Vec<ChatTurn> {
    let mut turns: Vec<ChatTurn> = Vec::new();

    for message in messages {
        if message.role == "user" {
            turns.push(ChatTurn {
                prompt: message.content.clone(),
                selected: message.selected.clone(),
                response: String::new(),
                sources: vec![],
                status: None,
                model: String::new(),
                elapsed_ms: None,
                is_error: false,
                attachments: message.attachments.clone(),
                timestamp: clock(message.created_at),
            });
        } else if let Some(turn) = turns.last_mut() {
            // Only the first answer after a question counts.
            if turn.response.is_empty() {
                turn.response = message.content.clone();
                turn.sources = message.sources.clone();
                turn.model = message.model.clone().unwrap_or_default();
                turn.elapsed_ms = message.elapsed_ms.map(|ms| ms.max(0) as u64);
                turn.is_error = message.is_error;
            }
        }
    }

    turns
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stored(role: &str, content: &str, at: i64) -> StoredMessage {
        StoredMessage {
            role: role.to_string(),
            content: content.to_string(),
            selected: None,
            sources: vec![],
            model: None,
            is_error: false,
            elapsed_ms: None,
            attachments: vec![],
            created_at: at,
        }
    }

    fn clock(ms: i64) -> String {
        format!("t{ms}")
    }

    #[test]
    fn a_question_and_its_answer_become_one_turn() {
        let mut answer = stored("assistant", "A language.", 2);
        answer.sources = vec![("Rust".into(), "https://rust-lang.org".into())];
        answer.model = Some("gemma".into());
        answer.elapsed_ms = Some(1500);

        let turns = turns_from_messages(&[stored("user", "/web what is rust?", 1), answer], clock);

        assert_eq!(turns.len(), 1);
        assert_eq!(turns[0].prompt, "/web what is rust?");
        assert_eq!(turns[0].response, "A language.");
        assert_eq!(turns[0].sources.len(), 1);
        assert_eq!(turns[0].model, "gemma");
        assert_eq!(turns[0].elapsed_ms, Some(1500));
        assert_eq!(turns[0].timestamp, "t1");
    }

    #[test]
    fn turns_keep_the_attachments_of_the_question() {
        let mut question = stored("user", "/screen what is this?", 1);
        question.attachments = vec!["/data/a.jpg".to_string()];
        let turns = turns_from_messages(&[question, stored("assistant", "A terminal.", 2)], clock);
        assert_eq!(turns[0].attachments, ["/data/a.jpg"]);
    }

    #[test]
    fn several_turns_keep_their_order() {
        let turns = turns_from_messages(
            &[
                stored("user", "one", 1),
                stored("assistant", "a1", 2),
                stored("user", "two", 3),
                stored("assistant", "a2", 4),
            ],
            clock,
        );
        assert_eq!(turns.len(), 2);
        assert_eq!(turns[1].prompt, "two");
        assert_eq!(turns[1].response, "a2");
    }

    #[test]
    fn a_question_without_an_answer_stays_as_an_empty_turn() {
        let turns = turns_from_messages(&[stored("user", "unanswered", 1)], clock);
        assert_eq!(turns.len(), 1);
        assert!(turns[0].response.is_empty());
    }

    #[test]
    fn an_error_answer_keeps_its_flag() {
        let mut failed = stored("assistant", "Error: busy", 2);
        failed.is_error = true;
        let turns = turns_from_messages(&[stored("user", "q", 1), failed], clock);
        assert!(turns[0].is_error);
    }

    #[test]
    fn an_answer_with_no_question_before_it_is_ignored() {
        let turns = turns_from_messages(&[stored("assistant", "orphan", 1)], clock);
        assert!(turns.is_empty());
    }
}
