pub mod anthropic;
pub mod openai;

use makepad_widgets::*;

use crate::model::{ChatMessage, Provider, ProviderConfig};

pub fn start_completion(cx: &mut Cx, config: &ProviderConfig, messages: &[ChatMessage]) {
    match config.provider {
        Provider::OpenAI => openai::start_stream(cx, config, messages),
        Provider::Anthropic => anthropic::start_stream(cx, config, messages),
    }
}
