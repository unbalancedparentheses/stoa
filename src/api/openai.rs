use futures::Stream;
use reqwest_eventsource::{Event, EventSource};
use std::pin::Pin;
use std::time::Duration;

use crate::api::{LlmEvent, TokenUsage};
use crate::model::{ChatMessage, Provider, ProviderConfig, Role};

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
        let is_ollama = config.provider == Provider::Ollama;
        if !is_ollama && config.api_key.is_empty() {
            yield LlmEvent::Error("API key not set. Go to Settings to configure.".into());
            return;
        }

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(if is_ollama { 120 } else { 30 }))
            .build()
            .unwrap_or_default();

        let mut body = serde_json::json!({
            "model": config.model,
            "messages": to_openai_messages(&messages, system_prompt.as_deref()),
            "stream": true,
            "stream_options": { "include_usage": true },
        });
        if let Some(t) = temperature {
            body["temperature"] = serde_json::json!(t);
        }
        if let Some(m) = max_tokens {
            body["max_completion_tokens"] = serde_json::json!(m);
        }

        let mut req = client
            .post(&config.api_url)
            .header("Content-Type", "application/json");
        if !is_ollama {
            req = req.header("Authorization", format!("Bearer {}", config.api_key));
        }
        let request = req.body(body.to_string());

        let mut es = match EventSource::new(request) {
            Ok(es) => es,
            Err(e) => {
                yield LlmEvent::Error(format!("Failed to connect: {e}"));
                return;
            }
        };

        let mut last_usage: Option<TokenUsage> = None;

        use futures::StreamExt;
        while let Some(event) = es.next().await {
            match event {
                Ok(Event::Open) => {}
                Ok(Event::Message(msg)) => {
                    if msg.data == "[DONE]" {
                        yield LlmEvent::Done(last_usage.take());
                        es.close();
                        break;
                    }
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&msg.data) {
                        // Extract token content
                        if let Some(content) = parsed["choices"][0]["delta"]["content"].as_str() {
                            if !content.is_empty() {
                                yield LlmEvent::Token(content.to_string());
                            }
                        }
                        // Extract usage from final chunk (OpenAI stream_options.include_usage)
                        if let Some(usage) = parsed.get("usage") {
                            if let (Some(pt), Some(ct)) = (
                                usage["prompt_tokens"].as_u64(),
                                usage["completion_tokens"].as_u64(),
                            ) {
                                last_usage = Some(TokenUsage {
                                    prompt_tokens: pt as u32,
                                    completion_tokens: ct as u32,
                                });
                            }
                        }
                    }
                }
                Err(reqwest_eventsource::Error::StreamEnded) => {
                    yield LlmEvent::Done(last_usage.take());
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
