pub mod anthropic;
pub mod ollama;
pub mod openai;

use futures::Stream;
use std::pin::Pin;

use crate::model::{ChatMessage, Provider, ProviderConfig};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
}

#[derive(Debug, Clone)]
pub enum LlmEvent {
    Token(String),
    Done(Option<TokenUsage>),
    Error(String),
}

pub fn new_shared_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .pool_max_idle_per_host(4)
        .build()
        .unwrap_or_default()
}

pub fn stream_completion(
    client: reqwest::Client,
    config: ProviderConfig,
    messages: Vec<ChatMessage>,
    system_prompt: Option<String>,
    temperature: Option<f64>,
    max_tokens: Option<u32>,
) -> Pin<Box<dyn Stream<Item = LlmEvent> + Send>> {
    match config.provider {
        Provider::OpenAI | Provider::Ollama | Provider::OpenRouter => openai::stream(client, config, messages, system_prompt, temperature, max_tokens),
        Provider::Anthropic => anthropic::stream(client, config, messages, system_prompt, temperature, max_tokens),
    }
}
