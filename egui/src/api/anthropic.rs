use futures::Stream;
use reqwest_eventsource::{Event, EventSource};
use std::pin::Pin;

use crate::api::LlmEvent;
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

pub fn stream(
    config: ProviderConfig,
    messages: Vec<ChatMessage>,
) -> Pin<Box<dyn Stream<Item = LlmEvent> + Send>> {
    Box::pin(async_stream::stream! {
        if config.api_key.is_empty() {
            yield LlmEvent::Error("Anthropic API key not set. Go to Settings to configure.".into());
            return;
        }

        let client = reqwest::Client::new();
        let body = serde_json::json!({
            "model": config.model,
            "messages": to_anthropic_messages(&messages),
            "max_tokens": 4096,
            "stream": true,
        });

        let request = client
            .post(&config.api_url)
            .header("x-api-key", &config.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .body(body.to_string());

        let mut es = EventSource::new(request).expect("failed to create event source");

        use futures::StreamExt;
        while let Some(event) = es.next().await {
            match event {
                Ok(Event::Open) => {}
                Ok(Event::Message(msg)) => {
                    match msg.event.as_str() {
                        "content_block_delta" => {
                            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&msg.data) {
                                if let Some(text) = parsed["delta"]["text"].as_str() {
                                    if !text.is_empty() {
                                        yield LlmEvent::Token(text.to_string());
                                    }
                                }
                            }
                        }
                        "message_stop" => {
                            yield LlmEvent::Done;
                            es.close();
                            break;
                        }
                        "error" => {
                            yield LlmEvent::Error(format!("Anthropic error: {}", msg.data));
                            es.close();
                            break;
                        }
                        _ => {}
                    }
                }
                Err(reqwest_eventsource::Error::StreamEnded) => {
                    yield LlmEvent::Done;
                    break;
                }
                Err(e) => {
                    yield LlmEvent::Error(format!("Anthropic stream error: {e}"));
                    es.close();
                    break;
                }
            }
        }
    })
}
