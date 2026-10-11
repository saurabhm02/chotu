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
}
