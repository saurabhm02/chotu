//! Constants and prompt templates. Environment variables are read in `services::llm`.

pub const DEFAULT_SYSTEM_PROMPT: &str = include_str!("../prompts/system_prompt.txt");
pub const WEB_PROMPT_TEMPLATE: &str = include_str!("../prompts/websearch_prompt.txt");

pub const BROWSER_USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) \
     AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

pub const DDG_URL: &str = "https://html.duckduckgo.com/html/";

pub const TIMEOUT_S: u64 = 5;
pub const PAGE_CONTENT_CHAR: usize = 1500;
pub const TOP_PAGE_K: usize = 5;

/// Vision models tried in this order when `AI_VISION_MODEL` is not set.
pub const DEFAULT_VISION_MODELS: [&str; 3] = [
    "nvidia/nemotron-3-nano-omni-30b-a3b-reasoning:free",
    "google/gemma-4-31b-it:free",
    "google/gemma-4-26b-a4b-it:free",
];

pub const ALL_MODELS_FAILED_MESSAGE: &str =
    "The AI service is not answering right now. Please try again in a minute.";

pub const WRONG_API_KEY_MESSAGE: &str = "The API key is incorrect.";
