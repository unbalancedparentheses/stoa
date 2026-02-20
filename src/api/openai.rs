use futures::Stream;
use reqwest_eventsource::{Event, EventSource};
use std::pin::Pin;
use std::time::Duration;

use crate::api::LlmEvent;
use crate::model::{ChatMessage, ProviderConfig, Role};

fn to_openai_messages(
    messages: &[ChatMessage],
    system_prompt: Option<&str>,
) -> Vec<serde_json::Value> {
    let mut out = Vec::new();

    if let Some(prompt) = system_prompt {
        if !prompt.is_empty() {
            out.push(serde_json::json!({
                "role": "system",
                "content": prompt,
            }));
        }
    }

    for m in messages {
        if m.streaming {
            continue;
        }
        out.push(serde_json::json!({
            "role": match m.role {
                Role::User => "user",
                Role::Assistant => "assistant",
            },
            "content": m.content,
        }));
    }

    out
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
            yield LlmEvent::Error("OpenAI API key not set. Go to Settings to configure.".into());
            return;
        }

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        let mut body = serde_json::json!({
            "model": config.model,
            "messages": to_openai_messages(&messages, system_prompt.as_deref()),
            "stream": true,
        });
        if let Some(t) = temperature {
            body["temperature"] = serde_json::json!(t);
        }
        if let Some(m) = max_tokens {
            body["max_completion_tokens"] = serde_json::json!(m);
        }

        let request = client
            .post(&config.api_url)
            .header("Authorization", format!("Bearer {}", config.api_key))
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
