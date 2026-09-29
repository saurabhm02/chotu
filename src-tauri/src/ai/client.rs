use async_openai::{
    config::OpenAIConfig,
    types::chat::{
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    },
    Client,
};
use std::env;

use crate::types::constants;

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
}
