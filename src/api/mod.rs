pub mod anthropic;
pub mod openai;

use futures::Stream;
use std::pin::Pin;

use crate::model::{ChatMessage, Provider, ProviderConfig};

#[derive(Debug, Clone)]
pub enum LlmEvent {
    Token(String),
    Done,
    Error(String),
}

pub fn stream_completion(
    config: ProviderConfig,
    messages: Vec<ChatMessage>,
    system_prompt: Option<String>,
    temperature: Option<f64>,
    max_tokens: Option<u32>,
) -> Pin<Box<dyn Stream<Item = LlmEvent> + Send>> {
    match config.provider {
        Provider::OpenAI => openai::stream(config, messages, system_prompt, temperature, max_tokens),
        Provider::Anthropic => anthropic::stream(config, messages, system_prompt, temperature, max_tokens),
    }
}
