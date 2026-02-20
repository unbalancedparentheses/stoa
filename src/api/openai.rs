use futures::Stream;
use reqwest_eventsource::{Event, EventSource};
use std::pin::Pin;

use crate::api::LlmEvent;
use crate::model::{ChatMessage, ProviderConfig, Role};

fn to_openai_messages(messages: &[ChatMessage]) -> Vec<serde_json::Value> {
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
            yield LlmEvent::Error("OpenAI API key not set. Go to Settings to configure.".into());
            return;
        }

        let client = reqwest::Client::new();
        let body = serde_json::json!({
            "model": config.model,
            "messages": to_openai_messages(&messages),
            "stream": true,
        });

        let request = client
            .post(&config.api_url)
            .header("Authorization", format!("Bearer {}", config.api_key))
            .header("Content-Type", "application/json")
            .body(body.to_string());

        let mut es = EventSource::new(request).expect("failed to create event source");

        use futures::StreamExt;
        while let Some(event) = es.next().await {
            match event {
                Ok(Event::Open) => {}
                Ok(Event::Message(msg)) => {
                    if msg.data == "[DONE]" {
                        yield LlmEvent::Done;
                        es.close();
                        break;
                    }
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&msg.data) {
                        if let Some(content) = parsed["choices"][0]["delta"]["content"].as_str() {
                            if !content.is_empty() {
                                yield LlmEvent::Token(content.to_string());
                            }
                        }
                    }
                }
                Err(reqwest_eventsource::Error::StreamEnded) => {
                    yield LlmEvent::Done;
                    break;
                }
                Err(e) => {
                    yield LlmEvent::Error(format!("OpenAI stream error: {e}"));
                    es.close();
                    break;
                }
            }
        }
    })
}
