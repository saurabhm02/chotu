const NO_TEXT_FOUND: &str = "[No text detected]";

pub fn format_extracted(texts: &[String]) -> String {
    let blocks: Vec<String> = texts
        .iter()
        .map(|text| text.trim())
        .filter(|text| !text.is_empty())
        .map(code_block)
        .collect();
    if blocks.is_empty() {
        return NO_TEXT_FOUND.to_string();
    }
    blocks.join("\n\n---\n\n")
}

fn code_block(text: &str) -> String {
    let mut fence = "```".to_string();

    while text.contains(&fence) {
        fence.push('`');
    }
    format!("{fence}\n{text}\n{fence}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_image_is_one_code_block() {
        let result = format_extracted(&["hello\nworld".to_string()]);
        assert_eq!(result, "```\nhello\nworld\n```");
    }

    #[test]
    fn several_images_are_separated_by_a_line() {
        let result = format_extracted(&["first".to_string(), "second".to_string()]);
        assert_eq!(result, "```\nfirst\n```\n\n---\n\n```\nsecond\n```");
    }

    #[test]
    fn an_image_with_no_text_is_skipped() {
        let result = format_extracted(&["  ".to_string(), "only this".to_string()]);
        assert_eq!(result, "```\nonly this\n```");
    }

    #[test]
    fn no_text_anywhere_says_so() {
        assert_eq!(format_extracted(&[]), "[No text detected]");
        assert_eq!(format_extracted(&["".to_string()]), "[No text detected]");
    }

    #[test]
    fn backticks_in_the_text_get_a_longer_fence() {
        let result = format_extracted(&["use ``` to start".to_string()]);
        assert!(result.starts_with("````\n"));
        assert!(result.ends_with("\n````"));
    }
}
