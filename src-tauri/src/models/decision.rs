use std::collections::HashMap;

use crate::config::{SURE_NO_THRESHOLD, SURE_YES_THRESHOLD};

/// How a yes/no question was answered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Yes,
    No,
    /// The model was not sure either way, so nobody should act on it.
    Unsure,
}

/// A verdict together with the number it came from.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Decision {
    pub verdict: Verdict,
    /// How likely "yes" is, from 0.0 (never) to 1.0 (certain).
    pub probability_of_yes: f32,
}

/// Turns the chance of "yes" into a verdict.
pub fn decide_verdict_from_probability(probability_of_yes: f32) -> Verdict {
    if probability_of_yes >= SURE_YES_THRESHOLD {
        Verdict::Yes
    } else if probability_of_yes <= SURE_NO_THRESHOLD {
        Verdict::No
    } else {
        Verdict::Unsure
    }
}

impl Decision {
    pub fn from_probability(probability_of_yes: f32) -> Self {
        Self {
            verdict: decide_verdict_from_probability(probability_of_yes),
            probability_of_yes,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChoiceOption {
    pub id: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuestionType {
    YesNo,
    Choice { options: Vec<ChoiceOption> },
    Score { levels: Vec<String> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionQuestion {
    pub id: String,
    pub instruction: String,
    pub question_type: QuestionType,
}

impl DecisionQuestion {
    pub fn yes_no(id: &str, instruction: &str) -> Self {
        Self {
            id: id.to_string(),
            instruction: instruction.to_string(),
            question_type: QuestionType::YesNo,
        }
    }

    pub fn choice(id: &str, instruction: &str, options: &[(&str, &str)]) -> Self {
        let options = options
            .iter()
            .map(|(option_id, desc)| ChoiceOption {
                id: option_id.to_string(),
                description: desc.to_string(),
            })
            .collect();

        Self {
            id: id.to_string(),
            instruction: instruction.to_string(),
            question_type: QuestionType::Choice { options },
        }
    }

    pub fn score(id: &str, instruction: &str, levels: &[&str]) -> Self {
        Self {
            id: id.to_string(),
            instruction: instruction.to_string(),
            question_type: QuestionType::Score {
                levels: levels.iter().map(|level| level.to_string()).collect(),
            },
        }
    }

    pub fn needs_full_model(&self) -> bool {
        self.question_type != QuestionType::YesNo
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChoiceAnswer {
    pub chosen: String,
    pub probabilities: Vec<(String, f32)>,
    pub confidence: f32,
}

impl ChoiceAnswer {
    pub fn probability_of(&self, option_id: &str) -> Option<f32> {
        self.probabilities
            .iter()
            .find(|(id, _)| id == option_id)
            .map(|(_, probability)| *probability)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScoreAnswer {
    pub value: f32,
    pub level: usize,
    pub label: String,
    pub probabilities: Vec<f32>,
    pub confidence: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Answer {
    YesNo(Decision),
    Choice(ChoiceAnswer),
    Score(ScoreAnswer),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Answers(HashMap<String, Answer>);

impl Answers {
    pub fn insert(&mut self, question_id: &str, answer: Answer) {
        self.0.insert(question_id.to_string(), answer);
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn yes_no(&self, question_id: &str) -> Option<Decision> {
        match self.0.get(question_id) {
            Some(Answer::YesNo(decision)) => Some(*decision),
            _ => None,
        }
    }

    pub fn choice(&self, question_id: &str) -> Option<&ChoiceAnswer> {
        match self.0.get(question_id) {
            Some(Answer::Choice(answer)) => Some(answer),
            _ => None,
        }
    }

    pub fn score(&self, question_id: &str) -> Option<&ScoreAnswer> {
        match self.0.get(question_id) {
            Some(Answer::Score(answer)) => Some(answer),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_high_chance_is_a_yes() {
        assert_eq!(decide_verdict_from_probability(0.97), Verdict::Yes);
    }

    #[test]
    fn a_low_chance_is_a_no() {
        assert_eq!(decide_verdict_from_probability(0.02), Verdict::No);
    }

    #[test]
    fn a_middle_chance_is_unsure() {
        assert_eq!(decide_verdict_from_probability(0.5), Verdict::Unsure);
    }

    #[test]
    fn the_edges_count_as_sure() {
        assert_eq!(
            decide_verdict_from_probability(SURE_YES_THRESHOLD),
            Verdict::Yes
        );
        assert_eq!(
            decide_verdict_from_probability(SURE_NO_THRESHOLD),
            Verdict::No
        );
    }

    #[test]
    fn a_decision_keeps_the_number_it_came_from() {
        let decision = Decision::from_probability(0.9);
        assert_eq!(decision.verdict, Verdict::Yes);
        assert_eq!(decision.probability_of_yes, 0.9);
    }

    #[test]
    fn question_builders_fill_in_the_type() {
        let question =
            DecisionQuestion::choice("kind", "What is it?", &[("a", "first"), ("b", "second")]);
        assert_eq!(question.id, "kind");
        assert!(question.needs_full_model());
        match question.question_type {
            QuestionType::Choice { options } => {
                assert_eq!(options.len(), 2);
                assert_eq!(options[1].id, "b");
                assert_eq!(options[1].description, "second");
            }
            other => panic!("expected a choice, got {other:?}"),
        }

        let score = DecisionQuestion::score("tone", "How polite?", &["rude", "ok", "polite"]);
        assert!(score.needs_full_model());
        assert!(!DecisionQuestion::yes_no("q", "Is it?").needs_full_model());
    }

    #[test]
    fn answers_are_found_only_by_their_own_kind() {
        let mut answers = Answers::default();
        answers.insert("yes", Answer::YesNo(Decision::from_probability(0.9)));
        assert_eq!(answers.len(), 1);
        assert!(answers.yes_no("yes").is_some());
        assert!(answers.choice("yes").is_none());
        assert!(answers.score("yes").is_none());
        assert!(answers.yes_no("missing").is_none());
    }

    #[test]
    fn a_choice_answer_gives_the_chance_of_each_option() {
        let answer = ChoiceAnswer {
            chosen: "a".to_string(),
            probabilities: vec![("a".to_string(), 0.8), ("b".to_string(), 0.2)],
            confidence: 0.7,
        };
        assert_eq!(answer.probability_of("b"), Some(0.2));
        assert_eq!(answer.probability_of("c"), None);
    }
}
