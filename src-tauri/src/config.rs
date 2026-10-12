//! Constants and prompt templates. Environment variables are read in `services::llm`.

pub const DEFAULT_SYSTEM_PROMPT: &str = include_str!("../prompts/system_prompt.txt");
pub const WEB_PROMPT_TEMPLATE: &str = include_str!("../prompts/websearch_prompt.txt");
pub const WEB_EMPTY_PROMPT_TEMPLATE: &str = include_str!("../prompts/websearch_empty_prompt.txt");
pub const TRANSLATE_PROMPT_TEMPLATE: &str = include_str!("../prompts/translate.txt");
pub const TLDR_PROMPT_TEMPLATE: &str = include_str!("../prompts/tldr.txt");
pub const BULLET_PROMPT_TEMPLATE: &str = include_str!("../prompts/bullets.txt");
pub const REFINE_PROMPT_TEMPLATE: &str = include_str!("../prompts/refine.txt");
pub const REWRITE_PROMPT_TEMPLATE: &str = include_str!("../prompts/rewrite.txt");
pub const EXPLAIN_PROMPT_TEMPLATE: &str = include_str!("../prompts/explain.txt");

pub const BROWSER_USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) \
     AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

pub const DDG_URL: &str = "https://html.duckduckgo.com/html/";

/// Where OpenRouter answers decision questions. This is not the chat address.
pub const DECISION_API_URL: &str = "https://openrouter.ai/api/alpha/decisions";

/// A decision model that has not answered after this long is given up on (the next one is tried).
pub const DECISION_TIMEOUT_S: u64 = 3;

/// Free decision models tried in this order when `AI_DECISION_MODEL` is not set.
/// Span-01 Lite goes first: it got every yes/no test question right. It only does yes/no.
pub const DEFAULT_DECISION_MODELS: [&str; 2] =
    ["respan/span-01-lite:free", "inception/mercury-decide:free"];

/// A chance of "yes" at or above this counts as a sure yes.
pub const SURE_YES_THRESHOLD: f32 = 0.8;

/// A chance of "yes" at or below this counts as a sure no.
pub const SURE_NO_THRESHOLD: f32 = 0.3;

pub const TIMEOUT_S: u64 = 5;
pub const PAGE_CONTENT_CHAR: usize = 1500;
pub const TOP_PAGE_K: usize = 5;
/// How much of the highlighted text is added to the web search words.
pub const SEARCH_SELECTED_CHARS: usize = 200;

/// A model that sends nothing for this long is given up on (the next model is tried).
pub const MODEL_TIMEOUT_S: u64 = 30;

/// Vision models tried in this order when `AI_VISION_MODEL` is not set.
pub const DEFAULT_VISION_MODELS: [&str; 3] = [
    "nvidia/nemotron-3-nano-omni-30b-a3b-reasoning:free",
    "google/gemma-4-31b-it:free",
    "google/gemma-4-26b-a4b-it:free",
];

pub const ALL_MODELS_FAILED_MESSAGE: &str =
    "The AI service is not answering right now. Please try again in a minute.";

pub const WRONG_API_KEY_MESSAGE: &str = "The API key is incorrect.";

/// Shown when a text command (like `/translate`) is sent with no text to work on.
pub const NO_TEXT_MESSAGE: &str = "Select some text, or type it after the command.";

pub const MAX_HISTORY_MESSAGES: usize = 10;
pub const MAX_HISTORY_MESSAGE_CHARS: usize = 4000;

pub const TITLE_PROMPT: &str = "Write a title of 3 to 6 words for the conversation below. \
Reply with the title only: no quotes, no full stop, no explanation.";

pub const TITLE_TIMEOUT_S: u64 = 20;

pub const CHECK_MISSING_QUESTION_MESSAGE: &str =
    "Type a yes/no question after the command, for example: /check is this a scam?";
pub const CHECK_MISSING_TEXT_MESSAGE: &str = "Highlight the text you want to check first.";
pub const DECISION_MODELS_UNAVAILABLE_MESSAGE: &str =
    "No answer right now. The free decision models may be busy, so try again in a minute.";

pub const AUTO_WEB_MIN_WORDS: usize = 2;
pub const AUTO_WEB_MAX_CHARS: usize = 400;
pub const AUTO_WEB_TIMEOUT_S: u64 = 3;

pub const AUTO_WEB_QUESTION: &str = "Does answering this question correctly need fresh information from the internet, such as today's news, current prices, live scores, the weather, recent events, or the latest version of something?";

/// The longest a call with several or harder questions (a scorecard) may take.
pub const DECISION_SLOW_TIMEOUT_S: u64 = 10;

/// Decision models that only answer yes/no. Choice and score questions skip them.
pub const NOUL_ONLY_DECISION_MODELS: [&str; 1] = ["respan/span-01-lite:free"];
