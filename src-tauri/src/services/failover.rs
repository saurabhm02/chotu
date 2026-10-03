//! Reading the list of models to try.

/// Turns the `.env` text "model-a, model-b" into a list. Blank or missing gives `defaults`.
pub fn model_list(setting: Option<&str>, defaults: &[&str]) -> Vec<String> {
    let from_setting: Vec<String> = setting
        .unwrap_or("")
        .split(',')
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty())
        .collect();

    if from_setting.is_empty() {
        defaults.iter().map(|name| name.to_string()).collect()
    } else {
        from_setting
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEFAULTS: [&str; 2] = ["a", "b"];

    #[test]
    fn missing_setting_uses_the_defaults() {
        assert_eq!(model_list(None, &DEFAULTS), vec!["a", "b"]);
    }

    #[test]
    fn blank_setting_uses_the_defaults() {
        assert_eq!(model_list(Some("  , ,"), &DEFAULTS), vec!["a", "b"]);
    }

    #[test]
    fn one_name_gives_a_list_of_one() {
        assert_eq!(model_list(Some("only-model"), &DEFAULTS), vec!["only-model"]);
    }

    #[test]
    fn setting_is_split_trimmed_and_kept_in_order() {
        assert_eq!(
            model_list(Some(" x/one:free ,y/two:free,, z/three "), &DEFAULTS),
            vec!["x/one:free", "y/two:free", "z/three"]
        );
    }
}
