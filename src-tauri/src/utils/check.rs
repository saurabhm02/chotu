use crate::config::{
    CHECK_MISSING_QUESTION_MESSAGE, CHECK_MISSING_TEXT_MESSAGE, DECISION_MODELS_UNAVAILABLE_MESSAGE,
};
use crate::models::decision::{Decision, Verdict};

const BAR_WIDTH: usize = 10;

#[derive(Debug, PartialEq)]
pub struct CheckInput<'a> {
    pub text: &'a str,
    pub question: &'a str,
}

pub fn parse_check_input<'a>(
    text: Option<&'a str>,
    question: &'a str,
) -> Result<CheckInput<'a>, &'static str> {
    let question = question.trim();
    let text = text.unwrap_or("").trim();

    if question.is_empty() {
        return Err(CHECK_MISSING_QUESTION_MESSAGE);
    }
    if text.is_empty() {
        return Err(CHECK_MISSING_TEXT_MESSAGE);
    }
    Ok(CheckInput { text, question })
}

pub fn format_response(decision: Option<Decision>) -> String {
    let Some(decision) = decision else {
        return DECISION_MODELS_UNAVAILABLE_MESSAGE.to_string();
    };

    let yes_probability = decision.probability_of_yes.clamp(0.0, 1.0);
    let no_probability = 1.0 - yes_probability;
    let (headline, bar_fill) = match decision.verdict {
        Verdict::Yes => (
            format!("**Yes**, {} sure", format_percent(yes_probability)),
            yes_probability,
        ),
        Verdict::No => (
            format!("**No**, {} sure", format_percent(no_probability)),
            no_probability,
        ),
        Verdict::Unsure => (
            format!(
                "**Not sure**, about {} yes",
                format_percent(yes_probability)
            ),
            yes_probability,
        ),
    };
    format!(
        "{headline}\n\n`{}`\n\n_A helper's guess, not a fact._",
        render_bar(bar_fill)
    )
}

/// 0.97 becomes "97%".
fn format_percent(probability: f32) -> String {
    format!("{}%", (probability * 100.0).round() as u32)
}

fn render_bar(fraction: f32) -> String {
    let filled = ((fraction * BAR_WIDTH as f32).round() as usize).min(BAR_WIDTH);
    format!("{}{}", "█".repeat(filled), "░".repeat(BAR_WIDTH - filled))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_and_question_are_trimmed() {
        let input = parse_check_input(Some("  my email  "), " is this a scam? ");
        assert_eq!(
            input,
            Ok(CheckInput {
                text: "my email",
                question: "is this a scam?",
            })
        );
    }

    #[test]
    fn a_missing_question_gives_the_question_hint() {
        assert_eq!(
            parse_check_input(Some("text"), "  "),
            Err(CHECK_MISSING_QUESTION_MESSAGE)
        );
    }

    #[test]
    fn missing_or_blank_text_gives_the_text_hint() {
        assert_eq!(
            parse_check_input(None, "is this rude?"),
            Err(CHECK_MISSING_TEXT_MESSAGE)
        );
        assert_eq!(
            parse_check_input(Some("   "), "is this rude?"),
            Err(CHECK_MISSING_TEXT_MESSAGE)
        );
    }

    #[test]
    fn a_sure_yes_shows_the_yes_probability() {
        let reply = format_response(Some(Decision::from_probability(0.97)));
        assert!(reply.starts_with("**Yes**, 97% sure"));
        assert!(reply.contains("`██████████`"));
    }

    #[test]
    fn a_sure_no_shows_how_sure_the_no_is() {
        let reply = format_response(Some(Decision::from_probability(0.02)));
        assert!(reply.starts_with("**No**, 98% sure"));
    }

    #[test]
    fn a_middle_answer_says_not_sure() {
        let reply = format_response(Some(Decision::from_probability(0.5)));
        assert!(reply.starts_with("**Not sure**, about 50% yes"));
        assert!(reply.contains("`█████░░░░░`"));
    }

    #[test]
    fn every_reply_says_it_is_a_guess() {
        for probability in [0.01, 0.5, 0.99] {
            let reply = format_response(Some(Decision::from_probability(probability)));
            assert!(reply.ends_with("_A helper's guess, not a fact._"));
        }
    }

    #[test]
    fn no_decision_gives_the_unavailable_message() {
        assert_eq!(format_response(None), DECISION_MODELS_UNAVAILABLE_MESSAGE);
    }

    #[test]
    fn the_bar_is_always_the_same_width() {
        for fraction in [0.0, 0.04, 0.5, 0.96, 1.0] {
            assert_eq!(render_bar(fraction).chars().count(), BAR_WIDTH);
        }
    }
}
