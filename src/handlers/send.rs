use iced::Task;

use crate::app::{ChatApp, Message};
use crate::model::Role;

impl ChatApp {
    pub(crate) fn handle_send_message(&mut self) -> Task<Message> {
        if self.input_value.trim().is_empty() || self.is_active_conv_streaming() { return Task::none(); }
        let mut text = self.input_value.clone();
        self.input_value.clear();
        self.error_message = None;
        self.model_picker_open = false;
        self.last_latency_ms = None;
        if let Some(context) = self.web_search_context.take() {
            text = format!("{context}{text}");
        }
        if let Some(content) = self.attached_file.take() {
            let filename = self.attached_filename.take().unwrap_or_default();
            text = format!("[Attached file: {filename}]\n```\n{content}\n```\n\n{text}");
        }
        let images = std::mem::take(&mut self.attached_images);
        let model_id = self.selected_model.clone();
        let Some(conv) = self.conversations.get_mut(self.active_conversation) else { return Task::none() };
        if images.is_empty() {
            conv.add_user_message(&text, Some(model_id.clone()));
        } else {
            conv.add_user_message_with_images(&text, Some(model_id.clone()), images);
        }
        if let Some(msg) = conv.messages.last_mut() {
            msg.token_count = Some(crate::cost::estimate_tokens(&text));
        }
        Self::handle_db_result(&mut self.error_message, crate::db::save_conversation(&self.db, conv));
        self.start_stream(&model_id)
    }

    pub(crate) fn handle_send_to_models(&mut self, model_ids: Vec<String>) -> Task<Message> {
        if self.input_value.trim().is_empty() || self.is_active_conv_streaming() || model_ids.is_empty() { return Task::none(); }
        let text = self.input_value.clone();
        self.input_value.clear();
        self.error_message = None;
        self.model_picker_open = false;
        self.last_latency_ms = None;
        let Some(conv) = self.conversations.get_mut(self.active_conversation) else { return Task::none() };
        conv.add_user_message(&text, None);
        if let Some(msg) = conv.messages.last_mut() { msg.token_count = Some(crate::cost::estimate_tokens(&text)); }
        Self::handle_db_result(&mut self.error_message, crate::db::save_conversation(&self.db, conv));
        self.start_multi_stream(&model_ids)
    }

    pub(crate) fn handle_send_to_all(&mut self) -> Task<Message> {
        let all_ids: Vec<String> = self.config.all_models().iter().map(|(_, id)| id.clone()).collect();
        if self.input_value.trim().is_empty() || self.is_active_conv_streaming() { return Task::none(); }
        let text = self.input_value.clone();
        self.input_value.clear();
        self.error_message = None;
        self.model_picker_open = false;
        self.last_latency_ms = None;
        let Some(conv) = self.conversations.get_mut(self.active_conversation) else { return Task::none() };
        conv.add_user_message(&text, None);
        if let Some(msg) = conv.messages.last_mut() { msg.token_count = Some(crate::cost::estimate_tokens(&text)); }
        Self::handle_db_result(&mut self.error_message, crate::db::save_conversation(&self.db, conv));
        self.start_multi_stream(&all_ids)
    }

    pub(crate) fn handle_retry_message(&mut self) -> Task<Message> {
        if self.is_active_conv_streaming() { return Task::none(); }
        let Some(conv) = self.conversations.get_mut(self.active_conversation) else { return Task::none() };
        let retry_model = conv.messages.last()
            .filter(|m| m.role == Role::Assistant)
            .and_then(|m| m.model.clone())
            .unwrap_or_else(|| self.selected_model.clone());
        if let Some(last) = conv.messages.last() { if last.role == Role::Assistant { conv.messages.pop(); } }
        if conv.messages.is_empty() { return Task::none(); }
        Self::handle_db_result(&mut self.error_message, crate::db::save_conversation(&self.db, conv));
        self.last_latency_ms = None;
        self.start_stream(&retry_model)
    }

    pub(crate) fn handle_review_with(&mut self, model_id: String) -> Task<Message> {
        let review_idx = self.review_picker.take();
        if self.is_active_conv_streaming() { return Task::none(); }
        let Some(conv) = self.conversations.get(self.active_conversation) else { return Task::none() };
        let review_content = review_idx.and_then(|idx| conv.messages.get(idx))
            .filter(|m| m.role == Role::Assistant).map(|m| m.content.clone()).unwrap_or_default();
        if review_content.is_empty() { return Task::none(); }
        let prompt = format!("[Review request]\nPlease review the following response and provide feedback, corrections, or improvements:\n\n{}", review_content);
        let Some(conv) = self.conversations.get_mut(self.active_conversation) else { return Task::none() };
        conv.add_user_message(&prompt, Some(model_id.clone()));
        Self::handle_db_result(&mut self.error_message, crate::db::save_conversation(&self.db, conv));
        self.last_latency_ms = None;
        self.start_stream(&model_id)
    }

    pub(crate) fn handle_analyze_with(&mut self, model_id: String) -> Task<Message> {
        let source_idx = match self.analyze_source_conversation.take() { Some(idx) => idx, None => return Task::none() };
        if self.is_active_conv_streaming() || source_idx >= self.conversations.len() { return Task::none(); }
        let source = &self.conversations[source_idx];
        let mut formatted = format!("[Analyze conversation] Analyzing: \"{}\"\n\n", source.title);
        for msg in &source.messages {
            let role_label = match msg.role {
                Role::User => "User",
                Role::Assistant => { match &msg.model { Some(m) => { formatted.push_str(&format!("Assistant ({m})")); "" } None => "Assistant" } }
            };
            if !role_label.is_empty() { formatted.push_str(role_label); }
            formatted.push_str(": ");
            formatted.push_str(&msg.content);
            formatted.push_str("\n\n");
        }
        formatted.push_str("Please analyze this conversation. Summarize the key points, identify any errors or areas for improvement, and provide your assessment.");
        let Some(conv) = self.conversations.get_mut(self.active_conversation) else { return Task::none() };
        conv.add_user_message(&formatted, Some(model_id.clone()));
        Self::handle_db_result(&mut self.error_message, crate::db::save_conversation(&self.db, conv));
        self.last_latency_ms = None;
        self.start_stream(&model_id)
    }
}
