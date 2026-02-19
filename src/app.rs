use iced::widget::{column, container, row};
use iced::{Element, Length, Subscription, Task, Theme, Color};

use crate::config::AppConfig;
use crate::model::{Conversation, Provider};
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
    ApplyPreset(String),
    SaveConfig,
    // Misc
    DismissError,
}

impl ChatApp {
    pub fn new() -> (Self, Task<Message>) {
        let config = AppConfig::load();
        let conversations = Conversation::load_all();
        let conversations = if conversations.is_empty() {
            vec![Conversation::new()]
        } else {
            conversations
        };

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

                let conv = &mut self.conversations[self.active_conversation];
                conv.add_user_message(&text);

                self.is_streaming = true;
                self.current_response.clear();

                let messages = conv.messages.clone();
                let provider_config = self.config.active_provider_config().clone();

                Task::run(
                    crate::api::stream_completion(provider_config, messages),
                    |event| match event {
                        crate::api::LlmEvent::Token(t) => Message::StreamToken(t),
                        crate::api::LlmEvent::Done => Message::StreamComplete,
                        crate::api::LlmEvent::Error(e) => Message::StreamError(e),
                    },
                )
            }
            Message::StreamToken(token) => {
                self.current_response.push_str(&token);
                let conv = &mut self.conversations[self.active_conversation];
                conv.update_assistant_streaming(&self.current_response);
                Task::none()
            }
            Message::StreamComplete => {
                self.is_streaming = false;
                let conv = &mut self.conversations[self.active_conversation];
                conv.finalize_assistant_message(&self.current_response);
                self.current_response.clear();
                conv.save();
                Task::none()
            }
            Message::StreamError(err) => {
                self.is_streaming = false;
                self.error_message = Some(err);
                self.current_response.clear();
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
                self.conversations.push(Conversation::new());
                self.active_conversation = self.conversations.len() - 1;
                self.view = View::Chat;
                Task::none()
            }
            Message::DeleteConversation(idx) => {
                if self.conversations.len() <= 1 {
                    return Task::none();
                }
                let conv = &self.conversations[idx];
                conv.delete();
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
            Message::ApplyPreset(preset) => {
                self.config.apply_preset(&preset);
                self.config_saved = false;
                Task::none()
            }
            Message::SaveConfig => {
                self.config.save();
                self.config_saved = true;
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
        Subscription::none()
    }
}
