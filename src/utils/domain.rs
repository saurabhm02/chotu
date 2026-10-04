//! Small helpers for showing a source: its site name, a coloured letter, a short model name.

/// "https://www.forbes.com/profile/x?a=1" -> "forbes.com"
pub fn host_of(url: &str) -> String {
    let no_scheme = url.split("://").last().unwrap_or(url);
    let host = no_scheme
        .split(|c| c == '/' || c == '?' || c == '#')
        .next()
        .unwrap_or("");
    host.strip_prefix("www.").unwrap_or(host).to_string()
}

/// First letter of the site, upper-case ("forbes.com" -> 'F').
pub fn initial_of(host: &str) -> char {
    host.chars()
        .find(|c| c.is_alphanumeric())
        .map(|c| c.to_ascii_uppercase())
        .unwrap_or('?')
}

const COLORS: [&str; 8] = [
    "#a7e08c", "#b9a6f0", "#f0a6c8", "#f2d57a", "#9ec5f4", "#f4b183", "#8fdcd0", "#c9c9d1",
];

/// The same site always gets the same colour.
pub fn avatar_color(host: &str) -> &'static str {
    let hash = host
        .bytes()
        .fold(0u32, |h, b| h.wrapping_mul(31).wrapping_add(b as u32));
    COLORS[hash as usize % COLORS.len()]
}

/// "nvidia/nemotron-3-nano-omni-30b-a3b-reasoning:free" -> "nemotron-3-nano-o..."
pub fn short_model(name: &str) -> String {
    let name = name.rsplit('/').next().unwrap_or(name);
    let name = name.split(':').next().unwrap_or(name);
    if name.chars().count() > 18 {
        let cut: String = name.chars().take(17).collect();
        format!("{cut}...")
    } else {
        name.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_drops_scheme_www_and_path() {
        assert_eq!(host_of("https://www.forbes.com/profile/elon?x=1"), "forbes.com");
        assert_eq!(host_of("http://en.wikipedia.org/wiki/Rust"), "en.wikipedia.org");
        assert_eq!(host_of("rust-lang.org"), "rust-lang.org");
    }

    #[test]
    fn initial_is_an_uppercase_letter() {
        assert_eq!(initial_of("forbes.com"), 'F');
        assert_eq!(initial_of(""), '?');
    }

    #[test]
    fn same_site_same_colour() {
        assert_eq!(avatar_color("forbes.com"), avatar_color("forbes.com"));
    }

    #[test]
    fn model_name_is_shortened() {
        assert_eq!(short_model("openrouter/free"), "free");
        assert_eq!(short_model("google/gemma-4-31b-it:free"), "gemma-4-31b-it");
        assert_eq!(
            short_model("nvidia/nemotron-3-nano-omni-30b-a3b-reasoning:free"),
            "nemotron-3-nano-o..."
        );
    }
}
