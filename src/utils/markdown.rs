use pulldown_cmark::{html, CodeBlockKind, Event, Options, Parser, Tag, TagEnd};

use super::highlight::highlight;

/// Markdown -> HTML. `[n]` becomes a clickable citation only when source n exists.
pub fn render_markdown_to_html(input: &str, source_count: usize) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(input, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, color_code_blocks(parser).into_iter());

    format_citations(&html_output, source_count)
}

/// Replaces every fenced code block with a coloured `<pre><code>` block.
fn color_code_blocks<'a>(parser: impl Iterator<Item = Event<'a>>) -> Vec<Event<'a>> {
    let mut events = Vec::new();
    // Some((language, code so far)) while we are inside a code block.
    let mut block: Option<(String, String)> = None;

    for event in parser {
        match event {
            Event::Start(Tag::CodeBlock(kind)) => {
                let lang = match kind {
                    CodeBlockKind::Fenced(lang) => lang.to_string(),
                    CodeBlockKind::Indented => String::new(),
                };
                block = Some((lang, String::new()));
            }
            Event::Text(text) if block.is_some() => {
                if let Some((_, code)) = block.as_mut() {
                    code.push_str(&text);
                }
            }
            Event::End(TagEnd::CodeBlock) => {
                if let Some((lang, code)) = block.take() {
                    let lang: String = lang.chars().filter(|c| c.is_alphanumeric()).collect();
                    let html = format!(
                        "<pre><code class=\"language-{lang}\">{}</code></pre>\n",
                        highlight(&code, &lang)
                    );
                    events.push(Event::Html(html.into()));
                }
            }
            other => events.push(other),
        }
    }
    events
}

fn format_citations(html: &str, source_count: usize) -> String {
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

            let in_range = citation_num
                .parse::<usize>()
                .is_ok_and(|n| n >= 1 && n <= source_count);

            if is_citation && in_range {
                result.push_str(&format!(
                    r#"<span class="citation-link" data-n="{citation_num}">[{citation_num}]</span>"#
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn citation_with_a_source_becomes_a_link() {
        let html = format_citations("<p>Rust is fast [2].</p>", 5);
        assert!(html.contains(r#"<span class="citation-link" data-n="2">[2]</span>"#));
    }

    #[test]
    fn citation_without_a_source_stays_plain() {
        assert_eq!(format_citations("<p>a [7] b</p>", 5), "<p>a [7] b</p>");
        assert_eq!(format_citations("<p>a [1] b</p>", 0), "<p>a [1] b</p>");
    }

    #[test]
    fn square_brackets_that_are_not_numbers_stay_plain() {
        assert_eq!(format_citations("<p>[abc] [x1]</p>", 5), "<p>[abc] [x1]</p>");
    }
}
