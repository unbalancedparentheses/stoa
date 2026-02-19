use makepad_widgets::*;

use crate::model::{ChatMessage, ProviderConfig, Role};

fn to_anthropic_messages(messages: &[ChatMessage]) -> Vec<serde_json::Value> {
    messages
        .iter()
        .filter(|m| !m.streaming)
        .map(|m| {
            serde_json::json!({
                "role": match m.role {
                    Role::User => "user",
                    Role::Assistant => "assistant",
                },
                "content": m.content,
            })
        })
        .collect()
}

pub fn start_stream(cx: &mut Cx, config: &ProviderConfig, messages: &[ChatMessage]) {
    let body = serde_json::json!({
        "model": config.model,
        "messages": to_anthropic_messages(messages),
        "max_tokens": 4096,
        "stream": true,
    });

    let mut request = HttpRequest::new(config.api_url.clone(), HttpMethod::POST);
    request.set_is_streaming();
    request.set_header("x-api-key".to_string(), config.api_key.clone());
    request.set_header("anthropic-version".to_string(), "2023-06-01".to_string());
    request.set_header("Content-Type".to_string(), "application/json".to_string());
    request.set_body(body.to_string().into_bytes());

    cx.http_request(live_id!(llm_stream), request);
}
