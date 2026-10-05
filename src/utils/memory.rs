use crate::models::chat::HistoryMessage;
use crate::models::{detect_cmd, ChatTurn};

const MAX_MESSAGES: usize = 10;

pub fn recent_messages(turns: &[ChatTurn]) -> Vec<HistoryMessage> {
    let mut messages = Vec::new();

    for turn in turns {
        if turn.is_error || turn.response.trim().is_empty() {
            continue;
        }
        let (_, question) = detect_cmd(&turn.prompt);
        let question = if question.is_empty() {
            turn.prompt.clone()
        } else {
            question
        };

        messages.push(HistoryMessage {
            role: "user".to_string(),
            content: question,
        });
        messages.push(HistoryMessage {
            role: "assistant".to_string(),
            content: turn.response.clone(),
        });
    }

    let skip = messages.len().saturating_sub(MAX_MESSAGES);
    messages.split_off(skip)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn turn(prompt: &str, response: &str, is_error: bool) -> ChatTurn {
        ChatTurn {
            prompt: prompt.to_string(),
            quote: None,
            response: response.to_string(),
            sources: vec![],
            status: None,
            model: String::new(),
            elapsed_ms: None,
            is_error,
            attachments: vec![],
            timestamp: String::new(),
        }
    }

    #[test]
    fn a_turn_becomes_a_question_and_an_answer() {
        let messages = recent_messages(&[turn("what is rust?", "A language.", false)]);
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "user");
        assert_eq!(messages[0].content, "what is rust?");
        assert_eq!(messages[1].role, "assistant");
        assert_eq!(messages[1].content, "A language.");
    }

    #[test]
    fn the_slash_command_is_removed_from_the_question() {
        let messages = recent_messages(&[turn("/web what is rust?", "A language.", false)]);
        assert_eq!(messages[0].content, "what is rust?");
    }

    #[test]
    fn a_lone_command_keeps_what_was_typed() {
        let messages = recent_messages(&[turn("/ss", "A terminal.", false)]);
        assert_eq!(messages[0].content, "/ss");
    }

    #[test]
    fn failed_and_empty_turns_are_left_out() {
        let messages = recent_messages(&[
            turn("one", "Error: busy", true),
            turn("two", "   ", false),
            turn("three", "ok", false),
        ]);
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].content, "three");
    }

    #[test]
    fn only_the_last_ten_messages_are_kept() {
        let turns: Vec<ChatTurn> = (0..8)
            .map(|i| turn(&format!("q{i}"), &format!("a{i}"), false))
            .collect();
        let messages = recent_messages(&turns);
        assert_eq!(messages.len(), 10);
        assert_eq!(messages[0].content, "q3");
        assert_eq!(messages[9].content, "a7");
    }
}
