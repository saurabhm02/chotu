use async_openai::{
    config::OpenAIConfig,
    types::chat::{
        ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
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
use std::time::Duration;
use tauri::ipc::Channel;

use crate::{
    config::{
        ALL_MODELS_FAILED_MESSAGE, DEFAULT_SYSTEM_PROMPT, DEFAULT_VISION_MODELS, MODEL_TIMEOUT_S,
        TITLE_PROMPT, TITLE_TIMEOUT_S, WEB_EMPTY_PROMPT_TEMPLATE, WEB_PROMPT_TEMPLATE,
        WRONG_API_KEY_MESSAGE,
    },
    models::chat::ChatMessage,
    models::stream::{Phase, StreamEvent},
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
        channel: &Channel<StreamEvent>,
        model: &str,
        req: CreateChatCompletionRequest,
        empty_err: &str,
        silent_phase: Phase,
    ) -> Result<String, String> {
        // A model that stays completely silent for this long is given up on.
        let silence_limit = Duration::from_secs(MODEL_TIMEOUT_S);
        let silence_err = format!("{model} sent nothing for {MODEL_TIMEOUT_S}s");

        let opened = tokio::time::timeout(silence_limit, self.client.chat().create_stream(req))
            .await
            .map_err(|_| silence_err.clone())?;
        let mut stream = opened.map_err(|e| {
            log::error!("LLM stream request failed: {e}");
            e.to_string()
        })?;

        let started = std::time::Instant::now();
        let mut full_text = String::new();
        let mut chunks = 0usize;
        let mut silent_pieces = 0usize;

        loop {
            // Wait for the next piece, but not forever.
            let next = match tokio::time::timeout(silence_limit, stream.next()).await {
                Ok(next) => next,
                Err(_) => {
                    log::warn!("{silence_err} (after {chunks} chunks)");
                    // Already part-way through an answer: keep what we have.
                    if full_text.is_empty() {
                        return Err(silence_err);
                    }
                    break;
                }
            };
            let Some(chunk) = next else { break }; // the stream ended normally
            let chunk = chunk.map_err(|e| {
                log::error!("LLM stream chunk error after {chunks} chunks: {e}");
                e.to_string()
            })?;
            let Some(delta) = chunk.choices.first().and_then(|c| c.delta.content.as_ref()) else {
                continue;
            };

            // Thinking models send empty pieces while they reason. Report it once.
            if delta.is_empty() {
                if full_text.is_empty() && silent_pieces == 0 {
                    let _ = channel.send(StreamEvent::Status(silent_phase));
                }
                silent_pieces += 1;
                continue;
            }

            if full_text.is_empty() {
                log::info!(
                    "first word after {:.1}s ({silent_pieces} silent pieces before it)",
                    started.elapsed().as_secs_f32()
                );
                // The router (`openrouter/free`) tells us which model really answered.
                let real_model = if chunk.model.is_empty() {
                    model.to_string()
                } else {
                    chunk.model.clone()
                };
                let _ = channel.send(StreamEvent::Model(real_model));
            }
            chunks += 1;
            full_text.push_str(delta);
            let _ = channel.send(StreamEvent::Token(delta.clone()));
        }

        if full_text.is_empty() {
            log::warn!("LLM returned no content ({silent_pieces} silent pieces)");
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
        channel: &Channel<StreamEvent>,
        model: &str,
        system_prompt: &str,
        history: &[ChatMessage],
        query: &str,
        image_urls: &[String],
        empty_err: &str,
    ) -> Result<String, String> {
        let mut messages: Vec<ChatCompletionRequestMessage> =
            vec![ChatCompletionRequestSystemMessageArgs::default()
                .content(system_prompt)
                .build()
                .map_err(|e| e.to_string())?
                .into()];
        messages.extend(history_messages(history)?);

        messages.push(
            ChatCompletionRequestUserMessageArgs::default()
                .content(user_content(query, image_urls))
                .build()
                .map_err(|e| e.to_string())?
                .into(),
        );

        let req = CreateChatCompletionRequestArgs::default()
            .model(model)
            .messages(messages)
            .stream(true)
            .build()
            .map_err(|e| e.to_string())?;

        // Means - if the image is not in the input only text we show Shrinking, and if image thent it says "Analyzing screenshot".
        let silent_phase = if image_urls.is_empty() {
            Phase::Thinking
        } else {
            Phase::Analyzing
        };
        self.stream_to_channel(channel, model, req, empty_err, silent_phase)
            .await
    }

    /// Tries the models in order and returns the first answer.
    /// Any error moves on to the next model, except 401: all models share one key.
    async fn ask_models(
        &self,
        channel: &Channel<StreamEvent>,
        models: &[String],
        system_prompt: &str,
        history: &[ChatMessage],
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
                .ask(
                    channel,
                    model,
                    system_prompt,
                    history,
                    query,
                    image_urls,
                    empty_err,
                )
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
        channel: &Channel<StreamEvent>,
        query: &str,
        context: Option<&str>,
        history: &[ChatMessage],
        image_paths: &[String],
    ) -> Result<String, String> {
        let message = prompt::build_message(query, context);

        let (models, image_urls) = if image_paths.is_empty() {
            (&self.text_models, Vec::new())
        } else {
            let _ = channel.send(StreamEvent::Status(Phase::Preparing));
            let urls = encode_images(image_paths).await?;
            let _ = channel.send(StreamEvent::Status(Phase::Analyzing));
            (&self.vision_models, urls)
        };

        self.ask_models(
            channel,
            models,
            DEFAULT_SYSTEM_PROMPT,
            history,
            &message,
            &image_urls,
            "empty text content response",
        )
        .await
    }

    pub async fn generate_title(&self, question: &str, answer: &str) -> Result<String, String> {
        let conversation = format!("Question:\n{question}\n\nAnswer:\n{answer}");

        for model in self.text_models.iter().take(2) {
            let messages: Vec<ChatCompletionRequestMessage> = vec![
                ChatCompletionRequestSystemMessageArgs::default()
                    .content(TITLE_PROMPT)
                    .build()
                    .map_err(|e| e.to_string())?
                    .into(),
                ChatCompletionRequestUserMessageArgs::default()
                    .content(conversation.clone())
                    .build()
                    .map_err(|e| e.to_string())?
                    .into(),
            ];
            let request = CreateChatCompletionRequestArgs::default()
                .model(model)
                .messages(messages)
                .build()
                .map_err(|e| e.to_string())?;

            let timeout = Duration::from_secs(TITLE_TIMEOUT_S);
            match tokio::time::timeout(timeout, self.client.chat().create(request)).await {
                Ok(Ok(response)) => {
                    let title = response
                        .choices
                        .first()
                        .and_then(|choice| choice.message.content.as_deref())
                        .map(str::trim)
                        .filter(|title| !title.is_empty());
                    if let Some(title) = title {
                        return Ok(title.to_string());
                    }
                    log::warn!("{model} returned an empty title");
                }
                Ok(Err(e)) => log::warn!("title request to {model} failed: {e}"),
                Err(_) => log::warn!("title request to {model} timed out after {TITLE_TIMEOUT_S}s"),
            }
        }

        Err("no title generated".to_string())
    }

    pub async fn call_llm_for_web(
        &self,
        channel: &Channel<StreamEvent>,
        query: &str,
        context: Option<&str>,
        history: &[ChatMessage],
        blocks: &str,
    ) -> Result<String, String> {
        let nonce = uuid::Uuid::new_v4().simple().to_string()[..12].to_string();
        let open = format!("<<<UNTRUSTED_WEB_CONTENT {nonce}>>>");
        let close = format!("<<<END_UNTRUSTED_WEB_CONTENT {nonce}>>>");

        let today = chrono::Utc::now().format("%A, %y-%m-%d").to_string();

        let prompt = if blocks.trim().is_empty() {
            WEB_EMPTY_PROMPT_TEMPLATE.replace("{TODAY}", &today)
        } else {
            WEB_PROMPT_TEMPLATE
                .replace("{TODAY}", &today)
                .replace("{OPEN_FENCE}", &open)
                .replace("{CLOSE_FENCE}", &close)
                .replace("{SOURCES}", blocks)
        };

        let sys_prompt = format!("{}\n{}", DEFAULT_SYSTEM_PROMPT, prompt);
        self.ask_models(
            channel,
            &self.text_models,
            &sys_prompt,
            history,
            &prompt::build_message(query, context),
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

fn history_messages(history: &[ChatMessage]) -> Result<Vec<ChatCompletionRequestMessage>, String> {
    let mut messages = Vec::new();
    for item in prompt::clean_history(history) {
        let message: ChatCompletionRequestMessage = if item.role == "assistant" {
            ChatCompletionRequestAssistantMessageArgs::default()
                .content(item.content)
                .build()
                .map_err(|e| e.to_string())?
                .into()
        } else {
            ChatCompletionRequestUserMessageArgs::default()
                .content(item.content)
                .build()
                .map_err(|e| e.to_string())?
                .into()
        };
        messages.push(message);
    }
    Ok(messages)
}
