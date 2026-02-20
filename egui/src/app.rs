use crate::api::LlmEvent;
use crate::config::AppConfig;
use crate::model::Conversation;
use crate::streaming::StreamBridge;
use crate::theme;
use crate::ui;

#[derive(Debug, Clone, PartialEq)]
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
    stream_bridge: StreamBridge,
}

impl ChatApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::apply_morphe_theme(&cc.egui_ctx);

        let config = AppConfig::load();
        let conversations = Conversation::load_all();
        let conversations = if conversations.is_empty() {
            vec![Conversation::new()]
        } else {
            conversations
        };

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
            stream_bridge: StreamBridge::new(),
        }
    }

    pub fn send_message(&mut self, ctx: &egui::Context) {
        if self.input_value.trim().is_empty() || self.is_streaming {
            return;
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

        self.stream_bridge
            .start(provider_config, messages, ctx.clone());
    }

    pub fn poll_stream(&mut self) {
        let events = self.stream_bridge.poll();
        for event in events {
            match event {
                LlmEvent::Token(token) => {
                    self.current_response.push_str(&token);
                    let conv = &mut self.conversations[self.active_conversation];
                    conv.update_assistant_streaming(&self.current_response);
                }
                LlmEvent::Done => {
                    self.is_streaming = false;
                    let conv = &mut self.conversations[self.active_conversation];
                    conv.finalize_assistant_message(&self.current_response);
                    self.current_response.clear();
                    conv.save();
                }
                LlmEvent::Error(err) => {
                    self.is_streaming = false;
                    self.error_message = Some(err);
                    self.current_response.clear();
                }
            }
        }
    }
}

impl eframe::App for ChatApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_stream();

        // Panel order: bottom → left → right → central
        ui::bottom_bar::draw(ctx, self);
        ui::sidebar::draw(ctx, self);
        ui::right_panel::draw(ctx, self);

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(theme::BG_MAIN))
            .show(ctx, |ui| match self.view {
                View::Chat => {
                    ui::chat_view::draw(ui, self);
                    ui::input_bar::draw(ui, self, ctx);
                }
                View::Settings => {
                    ui::settings::draw(ui, self);
                }
            });
    }
}
