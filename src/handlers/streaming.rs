use std::collections::HashMap;
use std::time::Instant;

use iced::Task;

use crate::app::{ActiveStream, ChatApp, Message, StreamId};
use crate::model::Role;

/// Generate a short conversation title using the LLM.
pub(crate) async fn generate_title(
    client: reqwest::Client,
    config: crate::model::ProviderConfig,
    user_msg: String,
    assistant_msg: String,
) -> String {
    let prompt = format!(
        "Generate a very short title (3-6 words, no quotes) for a conversation that starts with:\nUser: {}\nAssistant: {}",
        user_msg.chars().take(200).collect::<String>(),
        assistant_msg.chars().take(200).collect::<String>(),
    );

    let body = serde_json::json!({
        "model": config.model,
        "max_tokens": 30,
        "messages": [{"role": "user", "content": prompt}],
    });

    let mut req = client.post(&config.api_url)
        .header("Content-Type", "application/json");

    match config.provider {
        crate::model::Provider::Anthropic => {
            req = req.header("x-api-key", &config.api_key)
                .header("anthropic-version", "2023-06-01");
        }
        crate::model::Provider::Ollama => {}
        _ => {
            req = req.header("Authorization", format!("Bearer {}", config.api_key));
        }
    }

    let resp = match req.body(body.to_string()).send().await {
        Ok(r) => r,
        Err(_) => return String::new(),
    };

    let json: serde_json::Value = match resp.json().await {
        Ok(j) => j,
        Err(_) => return String::new(),
    };

    let title = match config.provider {
        crate::model::Provider::Anthropic => {
            json["content"][0]["text"].as_str().unwrap_or("").to_string()
        }
        _ => {
            json["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string()
        }
    };

    title.trim().trim_matches('"').chars().take(50).collect()
}

impl ChatApp {
    pub(crate) fn start_stream(&mut self, model_id: &str) -> Task<Message> {
        self.error_message = None;
        let Some(conv) = self.conversations.get_mut(self.active_conversation) else { return Task::none() };
        let conv_id = conv.id.clone();
        let msg_index = conv.push_streaming_assistant(Some(model_id.to_string()));
        let messages = conv.messages.clone();
        let provider_config = self.config.provider_config_for_model(model_id);
        let system_prompt = if !conv.system_prompt.is_empty() {
            Some(conv.system_prompt.clone())
        } else if !self.config.system_prompt.is_empty() {
            Some(self.config.system_prompt.clone())
        } else {
            None
        };
        let temperature = self.config.temperature.parse::<f64>().ok();
        let max_tokens = self.config.max_tokens.parse::<u32>().ok();

        let stream_id = self.next_stream_id;
        self.next_stream_id += 1;

        let (task, handle) = Task::run(
            crate::api::stream_completion(self.http_client.clone(), provider_config, messages, system_prompt, temperature, max_tokens),
            move |event| match event {
                crate::api::LlmEvent::Token(t) => Message::StreamToken(stream_id, t),
                crate::api::LlmEvent::Done(_usage) => Message::StreamComplete(stream_id),
                crate::api::LlmEvent::Error(e) => Message::StreamError(stream_id, e),
            },
        ).abortable();

        self.active_streams.insert(stream_id, ActiveStream {
            model: model_id.to_string(),
            current_response: String::new(),
            message_index: msg_index,
            conversation_id: conv_id,
            abort_handle: handle,
            stream_start: Instant::now(),
            first_token_received: false,
        });
        task
    }

    pub(crate) fn start_multi_stream(&mut self, model_ids: &[String]) -> Task<Message> {
        let tasks: Vec<Task<Message>> = model_ids.iter().map(|id| self.start_stream(id)).collect();
        Task::batch(tasks)
    }

    pub(crate) fn handle_stream_token(&mut self, id: StreamId, token: String) -> Task<Message> {
        if let Some(stream) = self.active_streams.get_mut(&id) {
            if !stream.first_token_received {
                stream.first_token_received = true;
                self.last_latency_ms = Some(stream.stream_start.elapsed().as_millis());
            }
            stream.current_response.push_str(&token);
            let idx = stream.message_index;
            let conv_id = stream.conversation_id.clone();
            if let Some(ci) = self.conv_index_by_id(&conv_id) {
                self.conversations[ci].append_streaming_token(idx, &token);
            }
        }
        Task::none()
    }

    pub(crate) fn handle_stream_complete(&mut self, id: StreamId) -> Task<Message> {
        if let Some(stream) = self.active_streams.remove(&id) {
            let latency = if stream.first_token_received {
                Some(stream.stream_start.elapsed().as_millis() as u64)
            } else {
                None
            };
            let ttfb = self.last_latency_ms.map(|ms| ms as u64);

            if let Some(ci) = self.conv_index_by_id(&stream.conversation_id) {
                let conv = &mut self.conversations[ci];
                conv.finalize_at(stream.message_index, &stream.current_response);
                if let Some(msg) = conv.messages.get_mut(stream.message_index) {
                    let tokens = crate::cost::estimate_tokens(&msg.content);
                    msg.token_count = Some(tokens);
                    msg.latency_ms = ttfb.or(latency);
                    let cost = crate::cost::message_cost(msg.model.as_deref().unwrap_or(""), &msg.role, tokens);
                    self.session_cost += cost;
                }
                let should_auto_title = conv.messages.iter().filter(|m| m.role == Role::Assistant && !m.streaming).count() == 1
                    && conv.title.chars().count() <= 30
                    && conv.forked_from.is_none();

                Self::handle_db_result(&mut self.error_message, crate::db::save_conversation(&self.db, conv));

                if should_auto_title {
                    let conv_id = conv.id.clone();
                    let user_msg = conv.messages.iter().find(|m| m.role == Role::User).map(|m| m.content.clone()).unwrap_or_default();
                    let assistant_msg = stream.current_response.clone();
                    let model = stream.model.clone();
                    let provider_config = self.config.provider_config_for_model(&model);
                    return Task::perform(
                        generate_title(self.http_client.clone(), provider_config, user_msg, assistant_msg),
                        move |title| Message::AutoTitleResult(conv_id.clone(), title),
                    );
                }
            }
        }
        Task::none()
    }

    pub(crate) fn handle_stream_error(&mut self, id: StreamId, err: String) -> Task<Message> {
        if let Some(stream) = self.active_streams.remove(&id) {
            let error_content = format!("[Error: {err}]");
            if let Some(ci) = self.conv_index_by_id(&stream.conversation_id) {
                let conv = &mut self.conversations[ci];
                conv.finalize_at(stream.message_index, &error_content);
                Self::handle_db_result(&mut self.error_message, crate::db::save_conversation(&self.db, conv));
            }
        }
        self.error_message = Some(err);
        Task::none()
    }

    pub(crate) fn handle_stop_streaming(&mut self) -> Task<Message> {
        let Some(active_conv) = self.conversations.get(self.active_conversation) else { return Task::none() };
        let active_conv_id = active_conv.id.clone();
        let all_streams: HashMap<StreamId, ActiveStream> = std::mem::take(&mut self.active_streams);
        let mut remaining = HashMap::new();
        let mut to_finalize = Vec::new();
        for (id, stream) in all_streams {
            if stream.conversation_id == active_conv_id {
                stream.abort_handle.abort();
                to_finalize.push(stream);
            } else {
                remaining.insert(id, stream);
            }
        }
        self.active_streams = remaining;
        if let Some(ci) = self.conv_index_by_id(&active_conv_id) {
            let conv = &mut self.conversations[ci];
            for stream in to_finalize {
                let content = if stream.current_response.is_empty() { "[stopped]".to_string() } else { stream.current_response };
                conv.finalize_at(stream.message_index, &content);
            }
            Self::handle_db_result(&mut self.error_message, crate::db::save_conversation(&self.db, conv));
        }
        Task::none()
    }

    pub(crate) fn handle_stop_stream(&mut self, id: StreamId) -> Task<Message> {
        if let Some(stream) = self.active_streams.remove(&id) {
            stream.abort_handle.abort();
            let content = if stream.current_response.is_empty() { "[stopped]".to_string() } else { stream.current_response };
            if let Some(ci) = self.conv_index_by_id(&stream.conversation_id) {
                let conv = &mut self.conversations[ci];
                conv.finalize_at(stream.message_index, &content);
                Self::handle_db_result(&mut self.error_message, crate::db::save_conversation(&self.db, conv));
            }
        }
        Task::none()
    }
}
