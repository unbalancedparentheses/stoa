use iced::widget::{column, container, row};
use iced::{keyboard, Element, Length, Subscription, Task, Theme, Color};
use rusqlite::Connection;
use std::time::Instant;

use crate::config::AppConfig;
use crate::model::{Conversation, Provider, Role};
use crate::ui;

#[derive(Debug, Clone)]
pub enum View {
    Chat,
    Settings,
}

pub struct ChatApp {
    pub conversations: Vec<Conversation>,
    pub active_conversation: usize,
    pub input_value: String,
    pub is_streaming: bool,
    pub current_response: String,
    pub config: AppConfig,
    pub view: View,
    pub error_message: Option<String>,
    pub config_saved: bool,
    // Rename
    pub renaming_conversation: Option<usize>,
    pub rename_value: String,
    // Latency
    pub stream_start: Option<Instant>,
    pub last_latency_ms: Option<u128>,
    // Abort streaming
    abort_handle: Option<iced::task::Handle>,
    // Database
    db: Connection,
    // Multi-model
    pub selected_model: String,
    pub streaming_model: Option<String>,
    pub model_picker_open: bool,
    pub review_picker: Option<usize>,
    pub analyze_source_conversation: Option<usize>,
}

#[derive(Debug, Clone)]
pub enum Message {
    // Input
    InputChanged(String),
    SendMessage,
    // Streaming
    StreamToken(String),
    StreamComplete,
    StreamError(String),
    StopStreaming,
    // Navigation
    SelectConversation(usize),
    NewConversation,
    DeleteConversation(usize),
    ShowSettings,
    ShowChat,
    // Settings
    SetProvider(Provider),
    SetApiKey(String),
    SetApiUrl(String),
    SetModel(String),
    SetSystemPrompt(String),
    SetTemperature(String),
    SetMaxTokens(String),
    ApplyPreset(String),
    SaveConfig,
    // Clipboard
    CopyToClipboard(String),
    // Rename
    StartRename(usize),
    RenameChanged(String),
    FinishRename,
    CancelRename,
    // Messages
    RetryMessage,
    DeleteMessage(usize),
    // Multi-model
    ToggleModelPicker,
    SelectModel(String),
    ShowReviewPicker(usize),
    DismissReviewPicker,
    ReviewWith(String),
    AnalyzeConversation(usize),
    AnalyzeWith(String),
    DismissAnalyzePicker,
    // Misc
    DismissError,
}

impl ChatApp {
    pub fn new() -> (Self, Task<Message>) {
        let config = AppConfig::load();
        let db = crate::db::open();
        let conversations = crate::db::load_all(&db);
        let conversations = if conversations.is_empty() {
            let c = Conversation::new();
            crate::db::save_conversation(&db, &c);
            vec![c]
        } else {
            conversations
        };

        let selected_model = config.active_provider_config().model.clone();
        (
            Self {
                conversations,
                active_conversation: 0,
                input_value: String::new(),
                is_streaming: false,
                current_response: String::new(),
                config,
                view: View::Chat,
                error_message: None,
                config_saved: false,
                renaming_conversation: None,
                rename_value: String::new(),
                stream_start: None,
                last_latency_ms: None,
                abort_handle: None,
                db,
                selected_model,
                streaming_model: None,
                model_picker_open: false,
                review_picker: None,
                analyze_source_conversation: None,
            },
            Task::none(),
        )
    }

    pub fn theme(&self) -> Theme {
        Theme::custom(
            "Morphe".to_string(),
            iced::theme::Palette {
                background: Color::from_rgb8(0x11, 0x19, 0x22),
                text: Color::from_rgb8(0xe6, 0xed, 0xf3),
                primary: Color::from_rgb8(0x4a, 0x9e, 0xc9),
                success: Color::from_rgb8(0x3f, 0xb8, 0x8c),
                warning: Color::from_rgb8(0xd4, 0xa5, 0x4e),
                danger: Color::from_rgb8(0xda, 0x6b, 0x6b),
            },
        )
    }

    fn start_stream(&mut self, model_id: &str) -> Task<Message> {
        self.is_streaming = true;
        self.current_response.clear();
        self.error_message = None;
        self.stream_start = Some(Instant::now());
        self.last_latency_ms = None;
        self.streaming_model = Some(model_id.to_string());

        let messages = self.conversations[self.active_conversation].messages.clone();
        let provider_config = self.config.provider_config_for_model(model_id);
        let system_prompt = if self.config.system_prompt.is_empty() {
            None
        } else {
            Some(self.config.system_prompt.clone())
        };
        let temperature = self.config.temperature.parse::<f64>().ok();
        let max_tokens = self.config.max_tokens.parse::<u32>().ok();

        let (task, handle) = Task::run(
            crate::api::stream_completion(provider_config, messages, system_prompt, temperature, max_tokens),
            |event| match event {
                crate::api::LlmEvent::Token(t) => Message::StreamToken(t),
                crate::api::LlmEvent::Done => Message::StreamComplete,
                crate::api::LlmEvent::Error(e) => Message::StreamError(e),
            },
        ).abortable();
        self.abort_handle = Some(handle);
        task
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::InputChanged(value) => {
                self.input_value = value;
                Task::none()
            }
            Message::SendMessage => {
                if self.input_value.trim().is_empty() || self.is_streaming {
                    return Task::none();
                }

                let text = self.input_value.clone();
                self.input_value.clear();
                self.error_message = None;

                let model_id = self.selected_model.clone();
                let conv = &mut self.conversations[self.active_conversation];
                conv.add_user_message(&text, Some(model_id.clone()));
                crate::db::save_conversation(&self.db, conv);

                self.start_stream(&model_id)
            }
            Message::StreamToken(token) => {
                if self.last_latency_ms.is_none() {
                    if let Some(start) = self.stream_start {
                        self.last_latency_ms = Some(start.elapsed().as_millis());
                    }
                }
                self.current_response.push_str(&token);
                let model = self.streaming_model.clone();
                let conv = &mut self.conversations[self.active_conversation];
                conv.update_assistant_streaming(&self.current_response, model);
                Task::none()
            }
            Message::StreamComplete => {
                self.is_streaming = false;
                self.stream_start = None;
                self.abort_handle = None;
                let model = self.streaming_model.take();
                let conv = &mut self.conversations[self.active_conversation];
                conv.finalize_assistant_message(&self.current_response, model);
                self.current_response.clear();
                crate::db::save_conversation(&self.db, conv);
                Task::none()
            }
            Message::StreamError(err) => {
                self.is_streaming = false;
                self.stream_start = None;
                self.abort_handle = None;
                self.streaming_model = None;
                self.error_message = Some(err);
                self.current_response.clear();
                Task::none()
            }
            Message::StopStreaming => {
                if let Some(handle) = self.abort_handle.take() {
                    handle.abort();
                }
                self.is_streaming = false;
                self.stream_start = None;
                let model = self.streaming_model.take();
                if !self.current_response.is_empty() {
                    let conv = &mut self.conversations[self.active_conversation];
                    conv.finalize_assistant_message(&self.current_response, model);
                    self.current_response.clear();
                    crate::db::save_conversation(&self.db, conv);
                }
                Task::none()
            }
            Message::SelectConversation(idx) => {
                if idx < self.conversations.len() {
                    self.active_conversation = idx;
                    self.view = View::Chat;
                }
                Task::none()
            }
            Message::NewConversation => {
                let conv = Conversation::new();
                crate::db::save_conversation(&self.db, &conv);
                self.conversations.push(conv);
                self.active_conversation = self.conversations.len() - 1;
                self.view = View::Chat;
                Task::none()
            }
            Message::DeleteConversation(idx) => {
                if self.conversations.len() <= 1 {
                    return Task::none();
                }
                let conv = &self.conversations[idx];
                crate::db::delete_conversation(&self.db, &conv.id);
                self.conversations.remove(idx);
                if self.active_conversation >= self.conversations.len() {
                    self.active_conversation = self.conversations.len() - 1;
                } else if self.active_conversation > idx {
                    self.active_conversation -= 1;
                }
                Task::none()
            }
            Message::ShowSettings => {
                self.view = View::Settings;
                self.config_saved = false;
                Task::none()
            }
            Message::ShowChat => {
                self.view = View::Chat;
                Task::none()
            }
            Message::SetProvider(provider) => {
                self.config.active_provider = provider;
                self.config_saved = false;
                Task::none()
            }
            Message::SetApiKey(key) => {
                self.config.active_provider_config_mut().api_key = key;
                self.config_saved = false;
                Task::none()
            }
            Message::SetApiUrl(url) => {
                self.config.active_provider_config_mut().api_url = url;
                self.config_saved = false;
                Task::none()
            }
            Message::SetModel(model) => {
                self.config.active_provider_config_mut().model = model;
                self.config_saved = false;
                Task::none()
            }
            Message::SetSystemPrompt(prompt) => {
                self.config.system_prompt = prompt;
                self.config_saved = false;
                Task::none()
            }
            Message::SetTemperature(val) => {
                self.config.temperature = val;
                self.config_saved = false;
                Task::none()
            }
            Message::SetMaxTokens(val) => {
                self.config.max_tokens = val;
                self.config_saved = false;
                Task::none()
            }
            Message::ApplyPreset(preset) => {
                self.config.apply_preset(&preset);
                self.selected_model = self.config.active_provider_config().model.clone();
                self.config_saved = false;
                Task::none()
            }
            Message::SaveConfig => {
                self.config.save();
                self.config_saved = true;
                Task::none()
            }
            Message::CopyToClipboard(content) => {
                iced::clipboard::write(content)
            }
            Message::StartRename(idx) => {
                self.rename_value = self.conversations[idx].title.clone();
                self.renaming_conversation = Some(idx);
                iced::widget::operation::focus("rename-input")
            }
            Message::RenameChanged(value) => {
                self.rename_value = value;
                Task::none()
            }
            Message::FinishRename => {
                if let Some(idx) = self.renaming_conversation.take() {
                    let new_title = self.rename_value.trim().to_string();
                    if !new_title.is_empty() && idx < self.conversations.len() {
                        self.conversations[idx].title = new_title;
                        let id = self.conversations[idx].id.clone();
                        crate::db::rename_conversation(
                            &self.db,
                            &id,
                            &self.conversations[idx].title,
                        );
                    }
                }
                self.rename_value.clear();
                Task::none()
            }
            Message::CancelRename => {
                self.renaming_conversation = None;
                self.rename_value.clear();
                self.model_picker_open = false;
                self.review_picker = None;
                self.analyze_source_conversation = None;
                Task::none()
            }
            Message::RetryMessage => {
                if self.is_streaming {
                    return Task::none();
                }
                let conv = &mut self.conversations[self.active_conversation];
                // Grab the model from the last assistant message before removing it
                let retry_model = conv.messages.last()
                    .filter(|m| m.role == Role::Assistant)
                    .and_then(|m| m.model.clone())
                    .unwrap_or_else(|| self.selected_model.clone());
                // Remove last assistant message if present
                if let Some(last) = conv.messages.last() {
                    if last.role == Role::Assistant {
                        conv.messages.pop();
                    }
                }
                if conv.messages.is_empty() {
                    return Task::none();
                }
                crate::db::save_conversation(&self.db, conv);

                self.start_stream(&retry_model)
            }
            Message::DeleteMessage(idx) => {
                let conv = &mut self.conversations[self.active_conversation];
                if idx < conv.messages.len() {
                    conv.messages.remove(idx);
                    crate::db::save_conversation(&self.db, conv);
                }
                Task::none()
            }
            Message::ToggleModelPicker => {
                self.model_picker_open = !self.model_picker_open;
                Task::none()
            }
            Message::SelectModel(model_id) => {
                self.selected_model = model_id;
                self.model_picker_open = false;
                Task::none()
            }
            Message::ShowReviewPicker(idx) => {
                self.review_picker = Some(idx);
                Task::none()
            }
            Message::DismissReviewPicker => {
                self.review_picker = None;
                Task::none()
            }
            Message::ReviewWith(model_id) => {
                let review_idx = self.review_picker.take();
                if self.is_streaming {
                    return Task::none();
                }
                // Get the specific assistant message to review
                let conv = &self.conversations[self.active_conversation];
                let review_content = review_idx
                    .and_then(|idx| conv.messages.get(idx))
                    .filter(|m| m.role == Role::Assistant)
                    .map(|m| m.content.clone())
                    .unwrap_or_default();
                if review_content.is_empty() {
                    return Task::none();
                }
                let prompt = format!(
                    "[Review request]\nPlease review the following response and provide feedback, corrections, or improvements:\n\n{}",
                    review_content
                );
                let conv = &mut self.conversations[self.active_conversation];
                conv.add_user_message(&prompt, Some(model_id.clone()));
                crate::db::save_conversation(&self.db, conv);
                self.start_stream(&model_id)
            }
            Message::AnalyzeConversation(idx) => {
                if idx < self.conversations.len() {
                    self.analyze_source_conversation = Some(idx);
                }
                Task::none()
            }
            Message::AnalyzeWith(model_id) => {
                let source_idx = match self.analyze_source_conversation.take() {
                    Some(idx) => idx,
                    None => return Task::none(),
                };
                if self.is_streaming || source_idx >= self.conversations.len() {
                    return Task::none();
                }
                // Format source conversation
                let source = &self.conversations[source_idx];
                let mut formatted = format!("[Analyze conversation] Analyzing: \"{}\"\n\n", source.title);
                for msg in &source.messages {
                    let role_label = match msg.role {
                        Role::User => "User",
                        Role::Assistant => {
                            match &msg.model {
                                Some(m) => { formatted.push_str(&format!("Assistant ({m})")); "" }
                                None => "Assistant",
                            }
                        }
                    };
                    if !role_label.is_empty() {
                        formatted.push_str(role_label);
                    }
                    formatted.push_str(": ");
                    formatted.push_str(&msg.content);
                    formatted.push_str("\n\n");
                }
                formatted.push_str("Please analyze this conversation. Summarize the key points, identify any errors or areas for improvement, and provide your assessment.");

                let conv = &mut self.conversations[self.active_conversation];
                conv.add_user_message(&formatted, Some(model_id.clone()));
                crate::db::save_conversation(&self.db, conv);
                self.start_stream(&model_id)
            }
            Message::DismissAnalyzePicker => {
                self.analyze_source_conversation = None;
                Task::none()
            }
            Message::DismissError => {
                self.error_message = None;
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let sidebar = ui::sidebar::view(self);
        let right_panel = ui::right_panel::view(self);
        let bottom_bar = ui::bottom_bar::view();

        let sep = |_: &Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(Color::from_rgb8(0x1e, 0x28, 0x34))),
            ..Default::default()
        };

        let content: Element<Message> = match self.view {
            View::Chat => {
                let chat = ui::chat_view::view(self);
                let input = ui::input_bar::view(self);
                column![
                    container(chat).height(Length::Fill),
                    input,
                ].into()
            }
            View::Settings => ui::settings::view(self),
        };

        let sep_v = || container(iced::widget::Space::new())
            .width(1).height(Length::Fill).style(sep);

        let main_row = row![
            sidebar,
            sep_v(),
            container(content).width(Length::Fill),
            sep_v(),
            right_panel,
        ];

        let layout = column![
            container(main_row).height(Length::Fill),
            bottom_bar,
        ];

        container(layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        iced::event::listen_with(|event, _status, _id| {
            if let iced::Event::Keyboard(keyboard::Event::KeyPressed {
                key, modifiers, ..
            }) = event
            {
                if modifiers.command() {
                    if let keyboard::Key::Character(ref c) = key {
                        if c.as_str() == "n" {
                            return Some(Message::NewConversation);
                        }
                    }
                }
                if let keyboard::Key::Named(keyboard::key::Named::Escape) = key {
                    return Some(Message::CancelRename);
                }
            }
            None
        })
    }
}
