//! Talking to the LLM: client setup (from env), request building, and streaming
//! the reply chunk-by-chunk to the frontend through a Tauri `Channel`.
use async_openai::{
    config::OpenAIConfig,
    types::chat::{
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequest, CreateChatCompletionRequestArgs,
    },
    Client,
};
use futures::StreamExt;
use std::env;
use std::sync::OnceLock;
use tauri::ipc::Channel;

use crate::config::{DEFAULT_SYSTEM_PROMPT, WEB_PROMPT_TEMPLATE};

pub struct AiClient {
    pub url: String,
    pub key: String,
    pub model: String,
    pub client: Client<OpenAIConfig>,
}

static SHARED: OnceLock<AiClient> = OnceLock::new();

impl AiClient {
    pub fn shared() -> &'static AiClient {
        SHARED.get_or_init(AiClient::new)
    }

    fn new() -> Self {
        let url = env::var("API_URL").expect("API_URL must be set");
        let key = env::var("API_KEY").expect("API_KEY must be set");
        let model = env::var("AI_MODEL").expect("AI_MODEL must be set");

        let config = OpenAIConfig::new()
            .with_api_base(url.clone())
            .with_api_key(key.clone());
        let client = Client::with_config(config);

        log::info!("AiClient created (url: {}, model: {})", url, model);
        Self {
            url,
            key,
            model,
            client,
        }
    }

    async fn stream_to_channel(
        &self,
        channel: &Channel<String>,
        req: CreateChatCompletionRequest,
        empty_err: &str,
    ) -> Result<String, String> {
        let mut stream = self.client.chat().create_stream(req).await.map_err(|e| {
            log::error!("LLM stream request failed: {e}");
            e.to_string()
        })?;

        let mut full_text = String::new();
        let mut chunks = 0usize;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| {
                log::error!("LLM stream chunk error after {chunks} chunks: {e}");
                e.to_string()
            })?;
            if let Some(delta) = chunk.choices.first().and_then(|c| c.delta.content.as_ref()) {
                chunks += 1;
                log::info!("chunk {chunks} at {:?}", std::time::Instant::now());
                full_text.push_str(delta);
                let _ = channel.send(delta.clone());
            }
        }

        if full_text.is_empty() {
            log::warn!("LLM returned no content ({chunks} chunks)");
            return Err(empty_err.to_string());
        }

        let preview: String = full_text.chars().take(200).collect();
        log::info!(
            "LLM response received: {} chars in {chunks} chunks, preview: {preview:?}",
            full_text.len()
        );

        Ok(full_text)
    }

    async fn ask(
        &self,
        channel: &Channel<String>,
        system_prompt: String,
        query: &str,
        empty_err: &str,
    ) -> Result<String, String> {
        let messages = vec![
            ChatCompletionRequestSystemMessageArgs::default()
                .content(system_prompt)
                .build()
                .map_err(|e| e.to_string())?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(query)
                .build()
                .map_err(|e| e.to_string())?
                .into(),
        ];

        let req = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages(messages)
            .stream(true)
            .build()
            .map_err(|e| e.to_string())?;

        self.stream_to_channel(channel, req, empty_err).await
    }

    /// Plain chat: default system prompt + the user's query.
    pub async fn call_llm(&self, channel: &Channel<String>, query: &str) -> Result<String, String> {
        log::info!(
            "sending streaming chat completion request to {} (model: {}), input: {:?}",
            self.url,
            self.model,
            query
        );
        self.ask(
            channel,
            DEFAULT_SYSTEM_PROMPT.to_string(),
            query,
            "empty text content response",
        )
        .await
    }

    /// Chat grounded on web sources (`blocks`), fenced as untrusted content.
    pub async fn call_llm_for_web(
        &self,
        channel: &Channel<String>,
        query: &str,
        blocks: &str,
    ) -> Result<String, String> {
        let nonce = uuid::Uuid::new_v4().simple().to_string()[..12].to_string();
        let open = format!("<<<UNTRUSTED_WEB_CONTENT {nonce}>>>");
        let close = format!("<<<END_UNTRUSTED_WEB_CONTENT {nonce}>>>");

        let today = chrono::Utc::now().format("%A, %y-%m-%d").to_string();

        let prompt = WEB_PROMPT_TEMPLATE
            .replace("{TODAY}", &today)
            .replace("{OPEN_FENCE}", &open)
            .replace("{CLOSE_FENCE}", &close)
            .replace("{SOURCES}", blocks);

        log::info!(
            "sending web chat completion request (model: {}), input: {:?}, sources: {} chars",
            self.model,
            query,
            blocks.len()
        );

        let sys_prompt = format!("{}\n{}", DEFAULT_SYSTEM_PROMPT, prompt);
        self.ask(channel, sys_prompt, query, "empty text response from llm!")
            .await
    }
}
