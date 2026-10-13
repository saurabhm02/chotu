use crate::config::{
    DECIDE_HINT_MESSAGE, DECIDE_MAX_OPTIONS, DECIDE_QUESTION, DECISION_MODELS_UNAVAILABLE_MESSAGE,
    SURE_CHOICE_THRESHOLD,
};
use crate::models::decision::{ChoiceAnswer, DecisionQuestion};
use crate::utils::check::{format_percent, render_bar};

pub const QUESTION_ID: &str = "pick";
const OPTION_IDS: [&str; DECIDE_MAX_OPTIONS] = ["a", "b", "c", "d"];

pub fn parse_options_reply(reply: &str, typed: &str) -> Result<Vec<String>, String> {
    let typed_words = split_into_words(typed);

    let mut options: Vec<String> = Vec::new();
    for line in reply.lines() {
        let option = strip_list_marker(line)
            .trim_matches(['"', '\'', '`'])
            .trim();
        let words = split_into_words(option);
        let written_by_user = !words.is_empty()
            && typed_words
                .windows(words.len())
                .any(|window| window == words.as_slice());
        let is_new = !options.iter().any(|seen| seen.eq_ignore_ascii_case(option));
        if written_by_user && is_new {
            options.push(option.to_string());
        }
    }

    if options.len() < 2 {
        return Err(DECIDE_HINT_MESSAGE.to_string());
    }
    if options.len() > DECIDE_MAX_OPTIONS {
        return Err(format!(
            "That is {} options, and I can compare up to {DECIDE_MAX_OPTIONS}.",
            options.len()
        ));
    }
    Ok(options)
}

/// "What should I eat, pizza?" becomes ["what", "should", "i", "eat", "pizza"].
fn split_into_words(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(|word| {
            word.trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase()
        })
        .filter(|word| !word.is_empty())
        .collect()
}

/// "- pizza", "* pizza" and "1. pizza" all become "pizza".
fn strip_list_marker(line: &str) -> &str {
    let line = line.trim().trim_start_matches(['-', '*', '•']).trim_start();
    let digits = line.chars().take_while(char::is_ascii_digit).count();
    match line[digits..].strip_prefix(['.', ')']) {
        Some(rest) if digits > 0 => rest.trim_start(),
        _ => line,
    }
}

pub fn build_question(options: &[String]) -> DecisionQuestion {
    let pairs: Vec<(&str, &str)> = OPTION_IDS
        .iter()
        .copied()
        .zip(options.iter().map(String::as_str))
        .collect();
    DecisionQuestion::choice(QUESTION_ID, DECIDE_QUESTION, &pairs)
}

pub fn format_choice(options: &[String], answer: Option<&ChoiceAnswer>) -> String {
    let Some(answer) = answer else {
        return DECISION_MODELS_UNAVAILABLE_MESSAGE.to_string();
    };

    let mut rows: Vec<(&str, &str, f32)> = OPTION_IDS
        .iter()
        .copied()
        .zip(options.iter().map(String::as_str))
        .map(|(id, label)| (id, label, answer.probability_of(id).unwrap_or(0.0)))
        .collect();
    rows.sort_by(|left, right| right.2.total_cmp(&left.2));

    let (_, chosen_label, chosen_probability) = rows
        .iter()
        .copied()
        .find(|(id, _, _)| *id == answer.chosen)
        .or_else(|| rows.first().copied())
        .unwrap_or(("", "", 0.0));

    let headline = if chosen_probability >= SURE_CHOICE_THRESHOLD {
        format!(
            "**{chosen_label}**, {} sure",
            format_percent(chosen_probability)
        )
    } else {
        format!(
            "**Not sure**, leaning towards **{chosen_label}** ({})",
            format_percent(chosen_probability)
        )
    };

    let width = rows
        .iter()
        .map(|(_, label, _)| label.chars().count())
        .max()
        .unwrap_or(0);
    let lines: Vec<String> = rows
        .iter()
        .map(|(_, label, probability)| {
            format!(
                "{label:<width$}  {} {}",
                render_bar(*probability),
                format_percent(*probability)
            )
        })
        .collect();

    format!(
        "{headline}\n\n```\n{}\n```\n\n_A helper's guess, not a fact._",
        lines.join("\n")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hint() -> Result<Vec<String>, String> {
        Err(DECIDE_HINT_MESSAGE.to_string())
    }

    #[test]
    fn a_clean_list_gives_the_options() {
        let reply = "pizza\nburger";
        assert_eq!(
            parse_options_reply(reply, "what should i eat pizza or burger?"),
            Ok(vec!["pizza".to_string(), "burger".to_string()])
        );
    }

    #[test]
    fn bullets_numbers_and_quotes_are_removed() {
        let reply = "- pizza\n2. burger\n* \"pasta\"\n4) salad";
        let typed = "pizza, burger, pasta or salad";
        assert_eq!(
            parse_options_reply(reply, typed),
            Ok(vec![
                "pizza".to_string(),
                "burger".to_string(),
                "pasta".to_string(),
                "salad".to_string()
            ])
        );
    }

    #[test]
    fn none_or_no_options_gives_the_hint() {
        assert_eq!(parse_options_reply("NONE", "what should i eat"), hint());
        assert_eq!(parse_options_reply("none", "what should i eat"), hint());
        assert_eq!(parse_options_reply("", "what should i eat"), hint());
    }

    #[test]
    fn one_option_gives_the_hint() {
        assert_eq!(parse_options_reply("pizza", "pizza"), hint());
    }

    #[test]
    fn an_option_the_user_did_not_write_is_dropped() {
        let reply = "pizza\nburger\nsandwich";
        assert_eq!(
            parse_options_reply(reply, "pizza or burger"),
            Ok(vec!["pizza".to_string(), "burger".to_string()])
        );
    }

    #[test]
    fn made_up_options_alone_give_the_hint() {
        let reply = "sandwich\nsalad";
        assert_eq!(parse_options_reply(reply, "what should i eat"), hint());
    }

    #[test]
    fn a_line_of_chat_is_dropped() {
        let reply = "Here are the options:\npizza\nburger";
        assert_eq!(
            parse_options_reply(reply, "pizza or burger"),
            Ok(vec!["pizza".to_string(), "burger".to_string()])
        );
    }

    #[test]
    fn a_single_letter_is_not_found_inside_another_word() {
        assert_eq!(parse_options_reply("e\nz", "pizza or burger"), hint());
    }

    #[test]
    fn capital_letters_and_punctuation_do_not_matter_for_the_check() {
        assert_eq!(
            parse_options_reply("Tea\nCoffee", "TEA or coffee?"),
            Ok(vec!["Tea".to_string(), "Coffee".to_string()])
        );
    }

    #[test]
    fn two_words_in_a_row_make_one_option() {
        assert_eq!(
            parse_options_reply("ice cream\ncake", "ice cream or cake"),
            Ok(vec!["ice cream".to_string(), "cake".to_string()])
        );
    }

    #[test]
    fn the_same_option_twice_counts_once() {
        assert_eq!(
            parse_options_reply("pizza\nPizza", "pizza or Pizza"),
            hint()
        );
    }

    #[test]
    fn more_than_four_options_says_so() {
        let error = parse_options_reply("a\nb\nc\nd\ne", "a or b or c or d or e").unwrap_err();
        assert!(error.contains("5 options"), "{error}");
    }

    fn answer(chosen: &str, chances: &[(&str, f32)]) -> ChoiceAnswer {
        ChoiceAnswer {
            chosen: chosen.to_string(),
            probabilities: chances
                .iter()
                .map(|(id, chance)| (id.to_string(), *chance))
                .collect(),
            confidence: 0.9,
        }
    }

    fn pizza_and_burger() -> Vec<String> {
        vec!["pizza".to_string(), "burger".to_string()]
    }

    #[test]
    fn the_question_gives_every_option_an_id() {
        let question = build_question(&pizza_and_burger());
        assert_eq!(question.id, QUESTION_ID);
        assert!(question.needs_full_model());
    }

    #[test]
    fn a_sure_pick_names_the_option_and_lists_every_chance() {
        let reply = format_choice(
            &pizza_and_burger(),
            Some(&answer("b", &[("a", 0.2), ("b", 0.8)])),
        );
        assert!(reply.starts_with("**burger**, 80% sure"), "{reply}");
        assert!(reply.contains("pizza   ██░░░░░░░░ 20%"), "{reply}");
        assert!(reply.contains("burger  ████████░░ 80%"), "{reply}");
        assert!(reply.ends_with("_A helper's guess, not a fact._"));
    }

    #[test]
    fn the_best_option_is_listed_first() {
        let reply = format_choice(
            &pizza_and_burger(),
            Some(&answer("b", &[("a", 0.2), ("b", 0.8)])),
        );
        assert!(reply.find("burger  ").unwrap() < reply.find("pizza   ").unwrap());
    }

    #[test]
    fn a_close_call_says_not_sure() {
        let reply = format_choice(
            &pizza_and_burger(),
            Some(&answer("a", &[("a", 0.52), ("b", 0.48)])),
        );
        assert!(
            reply.starts_with("**Not sure**, leaning towards **pizza** (52%)"),
            "{reply}"
        );
    }

    #[test]
    fn no_answer_gives_the_unavailable_message() {
        assert_eq!(
            format_choice(&pizza_and_burger(), None),
            DECISION_MODELS_UNAVAILABLE_MESSAGE
        );
    }
}
