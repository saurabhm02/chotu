use async_openai::{
    config::OpenAIConfig,
    types::chat::{
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    },
    Client,
};
use std::env;

use crate::types::constants::{self, DEFAULT_SYSTEM_PROMPT};

pub struct AiClient {
    pub url: String,
    pub key: String,
    pub model: String,
    pub client: Client<OpenAIConfig>,
}

impl AiClient {
    pub fn new() -> Self {
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

    pub async fn call_llm(&self, prompt: &str) -> Result<String, String> {
        let messages = vec![
            ChatCompletionRequestSystemMessageArgs::default()
                .content(constants::DEFAULT_SYSTEM_PROMPT)
                .build()
                .map_err(|e| e.to_string())?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(prompt)
                .build()
                .map_err(|e| e.to_string())?
                .into(),
        ];

        let req = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages(messages)
            .build()
            .map_err(|e| e.to_string())?;

        log::info!(
            "sending chat completion request to {} (model: {})",
            self.url,
            self.model
        );

        let response = self
            .client
            .chat()
            .create(req)
            .await
            .map_err(|e| e.to_string())?;

        log::info!("received chat completion response");

        let content = response
            .choices
            .first()
            .ok_or_else(|| "no choices".to_string())?
            .message
            .content
            .clone()
            .ok_or_else(|| "empty text content response".to_string())?;

        Ok(content)
    }
    pub async fn call_llm_for_web(&self, query: &str, blocks: &str) -> Result<String, String> {
        let nonce = uuid::Uuid::new_v4().simple().to_string()[..12].to_string();
        let open = format!("<<<UNTRUSTED_WEB_CONTENT {nonce}>>>");
        let close = format!("<<<END_UNTRUSTED_WEB_CONTENT {nonce}>>>");

        let today = chrono::Utc::now().format("%A, %y-%m-%d").to_string();

        let prompt = include_str!("../../prompts/websearch_prompt.txt")
            .replace("{TODAY}", &today)
            .replace("{OPEN_FENCE}", &open)
            .replace("{CLOSE_FENCE}", &close)
            .replace("{SOURCES}", blocks);

        let sys_prompt = format!("{}\n{}", DEFAULT_SYSTEM_PROMPT, prompt);
        let messages = vec![
            ChatCompletionRequestSystemMessageArgs::default()
                .content(sys_prompt)
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
            .build()
            .map_err(|e| e.to_string())?;

        let response = self
            .client
            .chat()
            .create(req)
            .await
            .map_err(|e| e.to_string())?;

        response
            .choices
            .first()
            .ok_or_else(|| "no choices".to_string())?
            .message
            .content
            .clone()
            .ok_or_else(|| "empty text response from llm!".to_string())
    }
}
