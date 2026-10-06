//! Colours code blocks: keywords, strings, comments and numbers.
//! Small on purpose: one pass over the text, no libraries.

const KEYWORDS: &[&str] = &[
    // Rust
    "fn", "let", "mut", "pub", "use", "mod", "struct", "enum", "impl", "trait", "match", "if",
    "else", "for", "while", "loop", "return", "const", "static", "async", "await", "move", "self",
    "Self", "as", "in", "where", "type", "ref", "true", "false", "None", "Some", "Ok", "Err",
    "break", "continue", "unsafe", "dyn",
    // JavaScript / TypeScript
    "function", "var", "class", "new", "import", "export", "from", "default", "try", "catch",
    "throw", "this", "null", "undefined", "typeof", "interface", "extends",
    // Python
    "def", "lambda", "with", "yield", "pass", "True", "False", "and", "or", "not", "is", "elif",
    "except", "finally", "raise", "global",
    // Shell and Go
    "echo", "cd", "then", "fi", "done", "do", "esac", "case", "func", "package", "range", "defer",
    "go", "chan", "select",
];

/// Languages where `#` starts a comment (the others use `//` and `/* */`).
const HASH_COMMENT_LANGS: &[&str] = &["py", "python", "sh", "bash", "shell", "zsh", "yaml", "yml", "toml", "rb", "ruby"];

fn escape(text: &str, out: &mut String) {
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(c),
        }
    }
}

fn span(class: &str, text: &str, out: &mut String) {
    out.push_str(&format!("<span class=\"{class}\">"));
    escape(text, out);
    out.push_str("</span>");
}

/// The code as HTML with `<span class="tok-...">` around the coloured parts.
pub fn highlight(code: &str, lang: &str) -> String {
    let chars: Vec<char> = code.chars().collect();
    let hash_comments = HASH_COMMENT_LANGS.contains(&lang);
    let mut out = String::with_capacity(code.len() * 2);
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        let next = chars.get(i + 1).copied();

        // Comments: to the end of the line, or /* ... */
        if (hash_comments && c == '#') || (!hash_comments && c == '/' && next == Some('/')) {
            let end = chars[i..].iter().position(|&x| x == '\n').map_or(chars.len(), |p| i + p);
            span("tok-comment", &chars[i..end].iter().collect::<String>(), &mut out);
            i = end;
        } else if !hash_comments && c == '/' && next == Some('*') {
            let mut end = i + 2;
            while end < chars.len() && !(chars[end - 1] == '*' && chars[end] == '/') {
                end += 1;
            }
            let end = (end + 1).min(chars.len());
            span("tok-comment", &chars[i..end].iter().collect::<String>(), &mut out);
            i = end;
        }
        // Strings: "text", 'text', `text`. A lone ' (like Rust's 'a) is not a string.
        else if c == '"' || c == '`' || (c == '\'' && is_quoted_char(&chars, i, lang)) {
            let mut end = i + 1;
            while end < chars.len() && chars[end] != c {
                if chars[end] == '\\' {
                    end += 1;
                }
                end += 1;
            }
            let end = (end + 1).min(chars.len());
            span("tok-string", &chars[i..end].iter().collect::<String>(), &mut out);
            i = end;
        }
        // Numbers
        else if c.is_ascii_digit() {
            let end = chars[i..]
                .iter()
                .position(|x| !(x.is_alphanumeric() || *x == '_' || *x == '.'))
                .map_or(chars.len(), |p| i + p);
            span("tok-number", &chars[i..end].iter().collect::<String>(), &mut out);
            i = end;
        }
        // Words: keywords and function calls
        else if c.is_alphabetic() || c == '_' {
            let end = chars[i..]
                .iter()
                .position(|x| !(x.is_alphanumeric() || *x == '_'))
                .map_or(chars.len(), |p| i + p);
            let word: String = chars[i..end].iter().collect();
            if KEYWORDS.contains(&word.as_str()) {
                span("tok-keyword", &word, &mut out);
            } else if chars.get(end) == Some(&'(') {
                span("tok-fn", &word, &mut out);
            } else {
                escape(&word, &mut out);
            }
            i = end;
        } else {
            escape(&c.to_string(), &mut out);
            i += 1;
        }
    }

    out
}

/// Is the `'` at `i` the start of a quoted text? In Rust `'a` is a lifetime, so only
/// `'x'` and `'\n'` count; in other languages every `'` starts a string.
fn is_quoted_char(chars: &[char], i: usize, lang: &str) -> bool {
    if lang != "rust" && lang != "rs" {
        return true;
    }
    chars.get(i + 2) == Some(&'\'') || (chars.get(i + 1) == Some(&'\\') && chars.get(i + 3) == Some(&'\''))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keywords_and_strings_get_classes() {
        let html = highlight("let name = \"Chotu\";", "rust");
        assert!(html.contains(r#"<span class="tok-keyword">let</span>"#));
        assert!(html.contains(r#"<span class="tok-string">"Chotu"</span>"#));
    }

    #[test]
    fn comments_run_to_the_end_of_the_line() {
        let html = highlight("a // note\nb", "rust");
        assert!(html.contains(r#"<span class="tok-comment">// note</span>"#));
        assert!(html.contains("\nb"));
    }

    #[test]
    fn hash_is_a_comment_in_python_only() {
        assert!(highlight("# hi", "python").contains("tok-comment"));
        assert!(!highlight("# hi", "rust").contains("tok-comment"));
    }

    #[test]
    fn rust_lifetime_is_not_a_string() {
        assert!(!highlight("fn f<'a>(x: &'a str)", "rust").contains("tok-string"));
    }

    #[test]
    fn html_characters_are_escaped() {
        let html = highlight("if a < b && c > d {}", "rust");
        assert!(html.contains("&lt;") && html.contains("&gt;") && html.contains("&amp;&amp;"));
        assert!(!html.contains("< b"));
    }

    #[test]
    fn numbers_and_calls_are_coloured() {
        let html = highlight("run(42)", "rust");
        assert!(html.contains(r#"<span class="tok-fn">run</span>"#));
        assert!(html.contains(r#"<span class="tok-number">42</span>"#));
    }

    #[test]
    fn unfinished_text_does_not_panic() {
        // An answer that is still streaming can stop anywhere.
        for text in ["\"abc", "/* abc", "'", "x = '\\", "`"] {
            let _ = highlight(text, "rust");
            let _ = highlight(text, "python");
        }
    }
}
