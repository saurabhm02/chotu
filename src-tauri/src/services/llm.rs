use async_openai::{
    config::OpenAIConfig,
    types::chat::{
        ChatCompletionRequestMessageContentPartImage, ChatCompletionRequestMessageContentPartText,
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        ChatCompletionRequestUserMessageContent, ChatCompletionRequestUserMessageContentPart,
        CreateChatCompletionRequest, CreateChatCompletionRequestArgs, ImageUrl,
    },
    Client,
};
use futures::StreamExt;
use std::env;
use std::path::Path;
use std::sync::OnceLock;
use tauri::ipc::Channel;

use crate::{
    config::{
        ALL_MODELS_FAILED_MESSAGE, DEFAULT_SYSTEM_PROMPT, DEFAULT_VISION_MODELS,
        WEB_PROMPT_TEMPLATE, WRONG_API_KEY_MESSAGE,
    },
    services::{failover, prompt},
    utils::images,
};

pub struct AiClient {
    pub url: String,
    pub key: String,
    pub text_models: Vec<String>,
    pub vision_models: Vec<String>,
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

        let text_models = failover::model_list(env::var("AI_MODEL").ok().as_deref(), &[]);
        assert!(!text_models.is_empty(), "AI_MODEL must be set");
        let vision_models = failover::model_list(
            env::var("AI_VISION_MODEL").ok().as_deref(),
            &DEFAULT_VISION_MODELS,
        );

        let config = OpenAIConfig::new()
            .with_api_base(url.clone())
            .with_api_key(key.clone());
        let client = Client::with_config(config);

        log::info!(
            "AiClient created (url: {url}, models: {text_models:?}, vision models: {vision_models:?})"
        );
        Self {
            url,
            key,
            text_models,
            vision_models,
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

    /// Sends one question (plus any images) to one model and streams the answer.
    async fn ask(
        &self,
        channel: &Channel<String>,
        model: &str,
        system_prompt: &str,
        query: &str,
        image_urls: &[String],
        empty_err: &str,
    ) -> Result<String, String> {
        let messages = vec![
            ChatCompletionRequestSystemMessageArgs::default()
                .content(system_prompt)
                .build()
                .map_err(|e| e.to_string())?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(user_content(query, image_urls))
                .build()
                .map_err(|e| e.to_string())?
                .into(),
        ];

        let req = CreateChatCompletionRequestArgs::default()
            .model(model)
            .messages(messages)
            .stream(true)
            .build()
            .map_err(|e| e.to_string())?;

        self.stream_to_channel(channel, req, empty_err).await
    }

    /// Tries the models in order and returns the first answer.
    /// Any error moves on to the next model, except 401: all models share one key.
    async fn ask_models(
        &self,
        channel: &Channel<String>,
        models: &[String],
        system_prompt: &str,
        query: &str,
        image_urls: &[String],
        empty_err: &str,
    ) -> Result<String, String> {
        for model in models {
            log::info!(
                "sending streaming chat completion request to {} (model: {model}, images: {}), input: {query:?}",
                self.url,
                image_urls.len()
            );

            let result = self
                .ask(channel, model, system_prompt, query, image_urls, empty_err)
                .await;

            match result {
                Ok(answer) => return Ok(answer),
                Err(error) => {
                    log::warn!("model {model} failed: {error}");
                    if error.contains("401") {
                        return Err(WRONG_API_KEY_MESSAGE.to_string());
                    }
                }
            }
        }

        Err(ALL_MODELS_FAILED_MESSAGE.to_string())
    }

    /// Plain chat. With images it uses the vision models, otherwise the text models.
    pub async fn call_llm(
        &self,
        channel: &Channel<String>,
        query: &str,
        context: Option<&str>,
        image_paths: &[String],
    ) -> Result<String, String> {
        let message = prompt::build_message(query, context);

        let (models, image_urls) = if image_paths.is_empty() {
            (&self.text_models, Vec::new())
        } else {
            (&self.vision_models, encode_images(image_paths).await?)
        };

        self.ask_models(
            channel,
            models,
            DEFAULT_SYSTEM_PROMPT,
            &message,
            &image_urls,
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

        let sys_prompt = format!("{}\n{}", DEFAULT_SYSTEM_PROMPT, prompt);
        self.ask_models(
            channel,
            &self.text_models,
            &sys_prompt,
            query,
            &[],
            "empty text response from llm!",
        )
        .await
    }
}

/// The user's message: plain text, or text followed by one part per image.
fn user_content(text: &str, image_urls: &[String]) -> ChatCompletionRequestUserMessageContent {
    if image_urls.is_empty() {
        return ChatCompletionRequestUserMessageContent::Text(text.to_string());
    }

    let mut parts = vec![ChatCompletionRequestUserMessageContentPart::Text(
        ChatCompletionRequestMessageContentPartText {
            text: text.to_string(),
            prompt_cache_breakpoint: None,
        },
    )];
    for url in image_urls {
        parts.push(ChatCompletionRequestUserMessageContentPart::ImageUrl(
            ChatCompletionRequestMessageContentPartImage {
                image_url: ImageUrl {
                    url: url.clone(),
                    detail: None,
                },
                prompt_cache_breakpoint: None,
            },
        ));
    }
    ChatCompletionRequestUserMessageContent::Array(parts)
}

/// Shrinking is heavy, so it runs on its own thread and the app stays responsive.
async fn encode_images(image_paths: &[String]) -> Result<Vec<String>, String> {
    let paths = image_paths.to_vec();
    tokio::task::spawn_blocking(move || {
        paths
            .iter()
            .map(|path| images::to_data_url(Path::new(path)))
            .collect::<Result<Vec<_>, _>>()
    })
    .await
    .map_err(|e| format!("image task failed: {e}"))?
}
