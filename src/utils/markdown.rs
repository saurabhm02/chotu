use pulldown_cmark::{html, Options, Parser};

/// Markdown -> HTML, with `[1]`-style citations wrapped in a styled span.
pub fn render_markdown_to_html(input: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(input, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    format_citations(&html_output)
}

fn format_citations(html: &str) -> String {
    let mut result = String::with_capacity(html.len() + 128);
    let mut chars = html.chars().peekable();
    let mut in_tag = false;

    while let Some(c) = chars.next() {
        if c == '<' {
            in_tag = true;
            result.push(c);
        } else if c == '>' {
            in_tag = false;
            result.push(c);
        } else if !in_tag && c == '[' {
            let mut citation_num = String::new();
            let mut is_citation = false;
            let mut temp = String::new();
            temp.push('[');

            while let Some(&next_c) = chars.peek() {
                if next_c.is_ascii_digit() {
                    citation_num.push(next_c);
                    temp.push(next_c);
                    chars.next();
                } else if next_c == ']' && !citation_num.is_empty() {
                    temp.push(']');
                    chars.next();
                    is_citation = true;
                    break;
                } else {
                    break;
                }
            }

            if is_citation {
                result.push_str(&format!(
                    r#"<span class="citation-tag">[{citation_num}]</span>"#
                ));
            } else {
                result.push_str(&temp);
            }
        } else {
            result.push(c);
        }
    }

    result
}
