use futures::Stream;
use reqwest_eventsource::{Event, EventSource};
use std::pin::Pin;
use std::time::Duration;

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
    system_prompt: Option<String>,
    temperature: Option<f64>,
    max_tokens: Option<u32>,
) -> Pin<Box<dyn Stream<Item = LlmEvent> + Send>> {
    Box::pin(async_stream::stream! {
        if config.api_key.is_empty() {
            yield LlmEvent::Error("Anthropic API key not set. Go to Settings to configure.".into());
            return;
        }

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        let mut body = serde_json::json!({
            "model": config.model,
            "messages": to_anthropic_messages(&messages),
            "max_tokens": max_tokens.unwrap_or(4096),
            "stream": true,
        });

        if let Some(t) = temperature {
            body["temperature"] = serde_json::json!(t);
        }

        if let Some(ref prompt) = system_prompt {
            if !prompt.is_empty() {
                body["system"] = serde_json::Value::String(prompt.clone());
            }
        }

        let request = client
            .post(&config.api_url)
            .header("x-api-key", &config.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .body(body.to_string());

        let mut es = match EventSource::new(request) {
            Ok(es) => es,
            Err(e) => {
                yield LlmEvent::Error(format!("Failed to connect: {e}"));
                return;
            }
        };

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
