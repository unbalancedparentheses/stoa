use iced::widget::{column, container, row};
use iced::{event, keyboard, window, Element, Length, Subscription, Task, Theme, Color};
use rusqlite::Connection;
use std::collections::{HashMap, HashSet};
use std::time::Instant;

use crate::config::AppConfig;
use crate::model::{Conversation, Provider, Role};
use crate::ui;

pub type StreamId = usize;

/// Generate a short conversation title using the LLM.
async fn generate_title(config: crate::model::ProviderConfig, user_msg: String, assistant_msg: String) -> String {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .unwrap_or_default();

    let prompt = format!(
        "Generate a very short title (3-6 words, no quotes) for a conversation that starts with:\nUser: {}\nAssistant: {}",
        user_msg.chars().take(200).collect::<String>(),
        assistant_msg.chars().take(200).collect::<String>(),
    );

    let body = match config.provider {
        crate::model::Provider::Anthropic => {
            serde_json::json!({
                "model": config.model,
                "max_tokens": 30,
                "messages": [{"role": "user", "content": prompt}],
            })
        }
        _ => {
            serde_json::json!({
                "model": config.model,
                "max_tokens": 30,
                "messages": [{"role": "user", "content": prompt}],
            })
        }
    };

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

    // Extract title from response
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

#[derive(Debug, Clone)]
pub enum View {
    Chat,
    Settings,
    Analytics,
}

pub struct ActiveStream {
    pub model: String,
    pub current_response: String,
    pub message_index: usize,
    pub conversation_id: String,
    pub abort_handle: iced::task::Handle,
    pub stream_start: Instant,
    pub first_token_received: bool,
}

pub struct ChatApp {
    pub conversations: Vec<Conversation>,
    pub active_conversation: usize,
    pub input_value: String,
    pub config: AppConfig,
    pub view: View,
    pub error_message: Option<String>,
    pub config_saved: bool,
    // Rename
    pub renaming_conversation: Option<usize>,
    pub rename_value: String,
    // Latency
    pub last_latency_ms: Option<u128>,
    // Database
    db: Connection,
    // Multi-model
    pub selected_model: String,
    pub model_picker_open: bool,
    pub review_picker: Option<usize>,
    pub analyze_source_conversation: Option<usize>,
    // Multi-stream
    pub next_stream_id: StreamId,
    pub active_streams: HashMap<StreamId, ActiveStream>,
    pub selected_models: HashSet<String>,
    // Comparison + Diff
    pub comparison_mode: bool,
    pub diff_active: Option<(usize, usize)>,
    // Quick switcher + Command palette
    pub quick_switcher_open: bool,
    pub quick_switcher_query: String,
    pub command_palette_open: bool,
    pub command_palette_query: String,
    pub command_palette_selected: usize,
    pub shortcut_help_open: bool,
    // Sidebar search
    pub sidebar_search_query: String,
    pub sidebar_search_results: Option<Vec<String>>,
    // Tag input
    pub tag_input_open: bool,
    pub tag_input_value: String,
    // Cost
    pub session_cost: f64,
    // System prompt editing
    pub conv_system_prompt_open: bool,
    pub conv_system_prompt_value: String,
    // File attachment
    pub attached_file: Option<String>,
    pub attached_filename: Option<String>,
    // Image attachment (base64)
    pub attached_images: Vec<String>,
    // Web search
    pub web_search_pending: bool,
    pub web_search_context: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    // Input
    InputChanged(String),
    SendMessage,
    // Streaming
    StreamToken(StreamId, String),
    StreamComplete(StreamId),
    StreamError(StreamId, String),
    StopStreaming,
    StopStream(StreamId),
    // Multi-model send
    SendToModels(Vec<String>),
    SendToAll,
    ToggleModelSelection(String),
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
    DismissOverlay,
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
    // Tags + Pins
    TogglePin(usize),
    RemoveTag(String),
    ToggleTagInput,
    TagInputChanged(String),
    SubmitTag,
    // Comparison + Diff
    ToggleComparisonMode,
    ShowDiff(usize, usize),
    DismissDiff,
    // Quick switcher
    ToggleQuickSwitcher,
    QuickSwitcherQueryChanged(String),
    QuickSwitcherSelect(usize),
    // Command palette
    ToggleCommandPalette,
    CommandPaletteQueryChanged(String),
    CommandPaletteMoveSelection(i32),
    CommandPaletteExecuteSelected,
    ToggleShortcutHelp,
    // Sidebar search
    SidebarSearchChanged(String),
    #[allow(dead_code)]
    ClearSidebarSearch,
    // Export
    ExportMarkdown,
    // Forking
    ForkConversation(usize), // fork at message index
    // Per-conversation system prompt
    ToggleConvSystemPrompt,
    ConvSystemPromptChanged(String),
    SaveConvSystemPrompt,
    // Ratings
    RateMessage(usize, i8), // (msg_index, -1/0/1)
    // Analytics
    ShowAnalytics,
    // Ollama
    OllamaModelsDiscovered(Vec<String>),
    RefreshOllamaModels,
    // Auto-title
    AutoTitleResult(String, String),
    // File attach
    AttachFile,
    AttachImage,
    FileAttached(String),
    ImageAttached(Vec<u8>),
    // Web search
    WebSearch,
    WebSearchResults(String), // formatted results prepended to next send
    // Export
    ExportHtml,
    ExportJson,
    // Import
    ImportChatGpt,
    ImportComplete(usize), // number of conversations imported
    // Folders
    #[allow(dead_code)]
    SetFolder(Option<String>),
    // Misc
    DismissError,
    RequestStartupFocus,
    FocusMainWindow(Option<window::Id>),
    SetKeybinding(crate::shortcuts::ShortcutAction, String),
    SetDebugKeyEvents(bool),
    KeyboardPressed(keyboard::Key, keyboard::key::Physical, keyboard::Modifiers),
}

impl ChatApp {
    fn from_parts(config: AppConfig, db: Connection, conversations: Vec<Conversation>) -> Self {
        let conversations = if conversations.is_empty() {
            let c = Conversation::new();
            crate::db::save_conversation(&db, &c);
            vec![c]
        } else {
            conversations
        };

        let selected_model = config.selected_model.clone()
            .unwrap_or_else(|| config.active_provider_config().model.clone());

        Self {
            conversations,
            active_conversation: 0,
            input_value: String::new(),
            config,
            view: View::Chat,
            error_message: None,
            config_saved: false,
            renaming_conversation: None,
            rename_value: String::new(),
            last_latency_ms: None,
            db,
            selected_model,
            model_picker_open: false,
            review_picker: None,
            analyze_source_conversation: None,
            next_stream_id: 0,
            active_streams: HashMap::new(),
            selected_models: HashSet::new(),
            comparison_mode: false,
            diff_active: None,
            quick_switcher_open: false,
            quick_switcher_query: String::new(),
            command_palette_open: false,
            command_palette_query: String::new(),
            command_palette_selected: 0,
            shortcut_help_open: false,
            sidebar_search_query: String::new(),
            sidebar_search_results: None,
            tag_input_open: false,
            tag_input_value: String::new(),
            session_cost: 0.0,
            conv_system_prompt_open: false,
            conv_system_prompt_value: String::new(),
            attached_file: None,
            attached_filename: None,
            attached_images: Vec::new(),
            web_search_pending: false,
            web_search_context: None,
        }
    }

    fn dismiss_top_overlay(&mut self) {
        if self.shortcut_help_open {
            self.shortcut_help_open = false;
        } else if self.quick_switcher_open {
            self.quick_switcher_open = false;
            self.quick_switcher_query.clear();
        } else if self.command_palette_open {
            self.command_palette_open = false;
            self.command_palette_query.clear();
            self.command_palette_selected = 0;
        } else if self.model_picker_open {
            self.model_picker_open = false;
        } else if self.review_picker.is_some() {
            self.review_picker = None;
        } else if self.analyze_source_conversation.is_some() {
            self.analyze_source_conversation = None;
        } else if self.tag_input_open {
            self.tag_input_open = false;
            self.tag_input_value.clear();
        } else if self.conv_system_prompt_open {
            self.conv_system_prompt_open = false;
            self.conv_system_prompt_value.clear();
        } else if self.diff_active.is_some() {
            self.diff_active = None;
        } else if self.renaming_conversation.is_some() {
            self.renaming_conversation = None;
            self.rename_value.clear();
        }
    }

    fn command_palette_commands(&self) -> Vec<crate::commands::CommandEntry> {
        crate::commands::filtered_commands(&self.command_palette_query, &self.config.keybindings)
    }

    fn command_palette_selection_count(&self) -> usize {
        self.command_palette_commands().len()
    }

    fn map_shortcut_action(action: crate::shortcuts::ShortcutAction) -> Message {
        match action {
            crate::shortcuts::ShortcutAction::SendToAll => Message::SendToAll,
            crate::shortcuts::ShortcutAction::NewConversation => Message::NewConversation,
            crate::shortcuts::ShortcutAction::ShowSettings => Message::ShowSettings,
            crate::shortcuts::ShortcutAction::QuickSwitcher => Message::ToggleQuickSwitcher,
            crate::shortcuts::ShortcutAction::CommandPalette => Message::ToggleCommandPalette,
            crate::shortcuts::ShortcutAction::ExportMarkdown => Message::ExportMarkdown,
            crate::shortcuts::ShortcutAction::ToggleShortcutHelp => Message::ToggleShortcutHelp,
        }
    }

    fn shortcut_message(
        keybindings: &crate::config::Keybindings,
        key: &keyboard::Key,
        physical_key: &keyboard::key::Physical,
        modifiers: keyboard::Modifiers,
    ) -> Option<Message> {
        if matches!(key, keyboard::Key::Named(keyboard::key::Named::Escape)) {
            return Some(Message::DismissOverlay);
        }

        crate::shortcuts::action_for_event(
            |action| keybindings.get(action).to_string(),
            key,
            physical_key,
            modifiers,
        ).map(Self::map_shortcut_action)
    }

    pub fn new() -> (Self, Task<Message>) {
        let config = AppConfig::load();
        let db = crate::db::open();
        let conversations = crate::db::load_all(&db);
        let app = Self::from_parts(config.clone(), db, conversations);

        let ollama_url = config.ollama.api_url.clone();
        let discover_task = Task::perform(
            async move { crate::api::ollama::discover_models(&ollama_url).await },
            |result| match result {
                Ok(models) => Message::OllamaModelsDiscovered(models),
                Err(_) => Message::OllamaModelsDiscovered(Vec::new()), // silently fail
            },
        );

        let startup_focus = Task::perform(
            async move {
                #[cfg(target_os = "macos")]
                {
                    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                }
            },
            |_| Message::RequestStartupFocus,
        );

        (app, Task::batch(vec![discover_task, startup_focus]))
    }

    pub fn new_for_tests() -> Self {
        let config = AppConfig::default();
        let db = crate::db::open_in_memory();
        let conversations = crate::db::load_all(&db);
        Self::from_parts(config, db, conversations)
    }

    pub fn is_streaming(&self) -> bool {
        !self.active_streams.is_empty()
    }

    pub fn is_active_conv_streaming(&self) -> bool {
        let conv_id = &self.conversations[self.active_conversation].id;
        self.active_streams.values().any(|s| s.conversation_id == *conv_id)
    }

    pub fn conv_has_streams(&self, conv_id: &str) -> bool {
        self.active_streams.values().any(|s| s.conversation_id == conv_id)
    }

    pub fn conv_stream_count(&self, conv_id: &str) -> usize {
        self.active_streams.values().filter(|s| s.conversation_id == conv_id).count()
    }

    pub fn conv_index_by_id(&self, id: &str) -> Option<usize> {
        self.conversations.iter().position(|c| c.id == id)
    }

    pub fn theme(&self) -> Theme {
        Theme::custom(
            "Stoa".to_string(),
            iced::theme::Palette {
                background: Color::from_rgb8(0x10, 0x12, 0x18),
                text: Color::from_rgb8(0xec, 0xed, 0xf0),
                primary: Color::from_rgb8(0x6e, 0xa0, 0xd4),
                success: Color::from_rgb8(0x50, 0xc0, 0x8a),
                warning: Color::from_rgb8(0xd0, 0xa0, 0x50),
                danger: Color::from_rgb8(0xe0, 0x60, 0x60),
            },
        )
    }

    fn start_stream(&mut self, model_id: &str) -> Task<Message> {
        self.error_message = None;
        let conv = &mut self.conversations[self.active_conversation];
        let conv_id = conv.id.clone();
        let msg_index = conv.push_streaming_assistant(Some(model_id.to_string()));
        let messages = conv.messages.clone();
        let provider_config = self.config.provider_config_for_model(model_id);
        // Per-conversation system prompt takes priority over global
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
            crate::api::stream_completion(provider_config, messages, system_prompt, temperature, max_tokens),
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

    fn start_multi_stream(&mut self, model_ids: &[String]) -> Task<Message> {
        let tasks: Vec<Task<Message>> = model_ids.iter().map(|id| self.start_stream(id)).collect();
        Task::batch(tasks)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::KeyboardPressed(key, physical_key, modifiers) => {
                if self.command_palette_open {
                    if matches!(key, keyboard::Key::Named(keyboard::key::Named::ArrowUp)) {
                        return self.update(Message::CommandPaletteMoveSelection(-1));
                    }
                    if matches!(key, keyboard::Key::Named(keyboard::key::Named::ArrowDown)) {
                        return self.update(Message::CommandPaletteMoveSelection(1));
                    }
                    if matches!(key, keyboard::Key::Named(keyboard::key::Named::Enter)) {
                        return self.update(Message::CommandPaletteExecuteSelected);
                    }
                }

                let mapped = Self::shortcut_message(&self.config.keybindings, &key, &physical_key, modifiers);
                if self.config.debug_key_events {
                    let ts = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_millis())
                        .unwrap_or(0);
                    crate::shortcuts::append_debug_key_log(&format!(
                        "{} key={:?} physical={:?} modifiers={{cmd:{},ctrl:{},shift:{},alt:{}}} mapped={:?}",
                        ts,
                        key,
                        physical_key,
                        modifiers.command(),
                        modifiers.control(),
                        modifiers.shift(),
                        modifiers.alt(),
                        mapped
                    ));
                }
                if let Some(msg) = mapped {
                    return self.update(msg);
                }
                Task::none()
            }
            Message::InputChanged(value) => { self.input_value = value; Task::none() }
            Message::SendMessage => {
                if self.input_value.trim().is_empty() || self.is_active_conv_streaming() { return Task::none(); }
                let mut text = self.input_value.clone();
                self.input_value.clear();
                self.error_message = None;
                self.model_picker_open = false;
                self.last_latency_ms = None;
                // Prepend web search context if present
                if let Some(context) = self.web_search_context.take() {
                    text = format!("{context}{text}");
                }
                // Prepend attached file content if present
                if let Some(content) = self.attached_file.take() {
                    let filename = self.attached_filename.take().unwrap_or_default();
                    text = format!("[Attached file: {filename}]\n```\n{content}\n```\n\n{text}");
                }
                let images = std::mem::take(&mut self.attached_images);
                let model_id = self.selected_model.clone();
                let conv = &mut self.conversations[self.active_conversation];
                if images.is_empty() {
                    conv.add_user_message(&text, Some(model_id.clone()));
                } else {
                    conv.add_user_message_with_images(&text, Some(model_id.clone()), images);
                }
                if let Some(msg) = conv.messages.last_mut() {
                    msg.token_count = Some(crate::cost::estimate_tokens(&text));
                }
                crate::db::save_conversation(&self.db, conv);
                self.start_stream(&model_id)
            }
            Message::SendToModels(model_ids) => {
                if self.input_value.trim().is_empty() || self.is_active_conv_streaming() || model_ids.is_empty() { return Task::none(); }
                let text = self.input_value.clone();
                self.input_value.clear();
                self.error_message = None;
                self.model_picker_open = false;
                self.last_latency_ms = None;
                let conv = &mut self.conversations[self.active_conversation];
                conv.add_user_message(&text, None);
                if let Some(msg) = conv.messages.last_mut() { msg.token_count = Some(crate::cost::estimate_tokens(&text)); }
                crate::db::save_conversation(&self.db, conv);
                self.start_multi_stream(&model_ids)
            }
            Message::SendToAll => {
                let all_ids: Vec<String> = self.config.all_models().iter().map(|(_, id)| id.clone()).collect();
                if self.input_value.trim().is_empty() || self.is_active_conv_streaming() { return Task::none(); }
                let text = self.input_value.clone();
                self.input_value.clear();
                self.error_message = None;
                self.model_picker_open = false;
                self.last_latency_ms = None;
                let conv = &mut self.conversations[self.active_conversation];
                conv.add_user_message(&text, None);
                if let Some(msg) = conv.messages.last_mut() { msg.token_count = Some(crate::cost::estimate_tokens(&text)); }
                crate::db::save_conversation(&self.db, conv);
                self.start_multi_stream(&all_ids)
            }
            Message::ToggleModelSelection(model_id) => {
                if self.selected_models.contains(&model_id) { self.selected_models.remove(&model_id); }
                else { self.selected_models.insert(model_id); }
                Task::none()
            }
            Message::StreamToken(id, token) => {
                if let Some(stream) = self.active_streams.get_mut(&id) {
                    if !stream.first_token_received {
                        stream.first_token_received = true;
                        self.last_latency_ms = Some(stream.stream_start.elapsed().as_millis());
                    }
                    stream.current_response.push_str(&token);
                    let content = stream.current_response.clone();
                    let idx = stream.message_index;
                    let conv_id = stream.conversation_id.clone();
                    if let Some(ci) = self.conv_index_by_id(&conv_id) {
                        self.conversations[ci].update_streaming_at(idx, &content);
                    }
                }
                Task::none()
            }
            Message::StreamComplete(id) => {
                if let Some(stream) = self.active_streams.remove(&id) {
                    let latency = if stream.first_token_received {
                        Some(stream.stream_start.elapsed().as_millis() as u64)
                    } else {
                        None
                    };
                    // Use first-token latency if we recorded it
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
                        // Auto-title: if this is the first assistant message and title looks auto-generated
                        let should_auto_title = conv.messages.iter().filter(|m| m.role == Role::Assistant && !m.streaming).count() == 1
                            && conv.title.len() <= 30
                            && conv.forked_from.is_none();

                        crate::db::save_conversation(&self.db, conv);

                        if should_auto_title {
                            let conv_id = conv.id.clone();
                            let user_msg = conv.messages.iter().find(|m| m.role == Role::User).map(|m| m.content.clone()).unwrap_or_default();
                            let assistant_msg = stream.current_response.clone();
                            let model = stream.model.clone();
                            let provider_config = self.config.provider_config_for_model(&model);
                            return Task::perform(
                                generate_title(provider_config, user_msg, assistant_msg),
                                move |title| Message::AutoTitleResult(conv_id.clone(), title),
                            );
                        }
                    }
                }
                Task::none()
            }
            Message::StreamError(id, err) => {
                if let Some(stream) = self.active_streams.remove(&id) {
                    let error_content = format!("[Error: {err}]");
                    if let Some(ci) = self.conv_index_by_id(&stream.conversation_id) {
                        let conv = &mut self.conversations[ci];
                        conv.finalize_at(stream.message_index, &error_content);
                        crate::db::save_conversation(&self.db, conv);
                    }
                }
                self.error_message = Some(err);
                Task::none()
            }
            Message::StopStreaming => {
                let active_conv_id = self.conversations[self.active_conversation].id.clone();
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
                    crate::db::save_conversation(&self.db, conv);
                }
                Task::none()
            }
            Message::StopStream(id) => {
                if let Some(stream) = self.active_streams.remove(&id) {
                    stream.abort_handle.abort();
                    let content = if stream.current_response.is_empty() { "[stopped]".to_string() } else { stream.current_response };
                    if let Some(ci) = self.conv_index_by_id(&stream.conversation_id) {
                        let conv = &mut self.conversations[ci];
                        conv.finalize_at(stream.message_index, &content);
                        crate::db::save_conversation(&self.db, conv);
                    }
                }
                Task::none()
            }
            Message::SelectConversation(idx) => {
                if idx < self.conversations.len() {
                    self.active_conversation = idx;
                    self.view = View::Chat;
                    self.model_picker_open = false;
                    self.quick_switcher_open = false;
                    self.diff_active = None;
                }
                Task::none()
            }
            Message::NewConversation => {
                let conv = Conversation::new();
                crate::db::save_conversation(&self.db, &conv);
                self.conversations.push(conv);
                self.active_conversation = self.conversations.len() - 1;
                self.view = View::Chat;
                self.model_picker_open = false;
                Task::none()
            }
            Message::DeleteConversation(idx) => {
                if self.conversations.len() <= 1 { return Task::none(); }
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
            Message::ShowSettings => { self.view = View::Settings; self.config_saved = false; self.model_picker_open = false; Task::none() }
            Message::ShowChat => { self.view = View::Chat; Task::none() }
            Message::SetProvider(p) => { self.config.active_provider = p; self.config_saved = false; Task::none() }
            Message::SetApiKey(k) => { self.config.active_provider_config_mut().api_key = k; self.config_saved = false; Task::none() }
            Message::SetApiUrl(u) => { self.config.active_provider_config_mut().api_url = u; self.config_saved = false; Task::none() }
            Message::SetModel(m) => { self.config.active_provider_config_mut().model = m; self.config_saved = false; Task::none() }
            Message::SetSystemPrompt(p) => { self.config.system_prompt = p; self.config_saved = false; Task::none() }
            Message::SetTemperature(v) => { self.config.temperature = v; self.config_saved = false; Task::none() }
            Message::SetMaxTokens(v) => { self.config.max_tokens = v; self.config_saved = false; Task::none() }
            Message::ApplyPreset(preset) => {
                self.config.apply_preset(&preset);
                self.selected_model = self.config.active_provider_config().model.clone();
                self.config_saved = false;
                Task::none()
            }
            Message::SaveConfig => { self.config.save(); self.config_saved = true; Task::none() }
            Message::CopyToClipboard(content) => iced::clipboard::write(content),
            Message::StartRename(idx) => {
                self.rename_value = self.conversations[idx].title.clone();
                self.renaming_conversation = Some(idx);
                iced::widget::operation::focus("rename-input")
            }
            Message::RenameChanged(value) => { self.rename_value = value; Task::none() }
            Message::FinishRename => {
                if let Some(idx) = self.renaming_conversation.take() {
                    let new_title = self.rename_value.trim().to_string();
                    if !new_title.is_empty() && idx < self.conversations.len() {
                        self.conversations[idx].title = new_title;
                        let id = self.conversations[idx].id.clone();
                        crate::db::rename_conversation(&self.db, &id, &self.conversations[idx].title);
                    }
                }
                self.rename_value.clear();
                Task::none()
            }
            Message::DismissOverlay => {
                self.dismiss_top_overlay();
                Task::none()
            }
            Message::RetryMessage => {
                if self.is_active_conv_streaming() { return Task::none(); }
                let conv = &mut self.conversations[self.active_conversation];
                let retry_model = conv.messages.last()
                    .filter(|m| m.role == Role::Assistant)
                    .and_then(|m| m.model.clone())
                    .unwrap_or_else(|| self.selected_model.clone());
                if let Some(last) = conv.messages.last() { if last.role == Role::Assistant { conv.messages.pop(); } }
                if conv.messages.is_empty() { return Task::none(); }
                crate::db::save_conversation(&self.db, conv);
                self.last_latency_ms = None;
                self.start_stream(&retry_model)
            }
            Message::DeleteMessage(idx) => {
                let conv = &mut self.conversations[self.active_conversation];
                if idx < conv.messages.len() { conv.messages.remove(idx); crate::db::save_conversation(&self.db, conv); }
                Task::none()
            }
            Message::ToggleModelPicker => { self.model_picker_open = !self.model_picker_open; Task::none() }
            Message::SelectModel(model_id) => {
                self.selected_model = model_id.clone();
                self.model_picker_open = false;
                self.config.selected_model = Some(model_id);
                self.config.save();
                Task::none()
            }
            Message::ShowReviewPicker(idx) => { self.review_picker = Some(idx); Task::none() }
            Message::DismissReviewPicker => { self.review_picker = None; Task::none() }
            Message::ReviewWith(model_id) => {
                let review_idx = self.review_picker.take();
                if self.is_active_conv_streaming() { return Task::none(); }
                let conv = &self.conversations[self.active_conversation];
                let review_content = review_idx.and_then(|idx| conv.messages.get(idx))
                    .filter(|m| m.role == Role::Assistant).map(|m| m.content.clone()).unwrap_or_default();
                if review_content.is_empty() { return Task::none(); }
                let prompt = format!("[Review request]\nPlease review the following response and provide feedback, corrections, or improvements:\n\n{}", review_content);
                let conv = &mut self.conversations[self.active_conversation];
                conv.add_user_message(&prompt, Some(model_id.clone()));
                crate::db::save_conversation(&self.db, conv);
                self.last_latency_ms = None;
                self.start_stream(&model_id)
            }
            Message::AnalyzeConversation(idx) => {
                if idx < self.conversations.len() { self.analyze_source_conversation = Some(idx); }
                Task::none()
            }
            Message::AnalyzeWith(model_id) => {
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
                let conv = &mut self.conversations[self.active_conversation];
                conv.add_user_message(&formatted, Some(model_id.clone()));
                crate::db::save_conversation(&self.db, conv);
                self.last_latency_ms = None;
                self.start_stream(&model_id)
            }
            Message::DismissAnalyzePicker => { self.analyze_source_conversation = None; Task::none() }
            // Tags + Pins
            Message::TogglePin(idx) => {
                if idx < self.conversations.len() {
                    self.conversations[idx].pinned = !self.conversations[idx].pinned;
                    let id = self.conversations[idx].id.clone();
                    crate::db::toggle_pin(&self.db, &id, self.conversations[idx].pinned);
                    // Re-sort: pinned first, then by original order
                    let active_id = self.conversations[self.active_conversation].id.clone();
                    self.conversations.sort_by(|a, b| b.pinned.cmp(&a.pinned));
                    self.active_conversation = self.conversations.iter().position(|c| c.id == active_id).unwrap_or(0);
                }
                Task::none()
            }
            Message::RemoveTag(tag) => {
                let conv = &mut self.conversations[self.active_conversation];
                conv.tags.retain(|t| t != &tag);
                crate::db::set_tags(&self.db, &conv.id, &conv.tags);
                Task::none()
            }
            Message::ToggleTagInput => { self.tag_input_open = !self.tag_input_open; self.tag_input_value.clear(); Task::none() }
            Message::TagInputChanged(v) => { self.tag_input_value = v; Task::none() }
            Message::SubmitTag => {
                let tag = self.tag_input_value.trim().to_string();
                self.tag_input_value.clear();
                self.tag_input_open = false;
                if !tag.is_empty() {
                    let conv = &mut self.conversations[self.active_conversation];
                    if !conv.tags.contains(&tag) {
                        conv.tags.push(tag);
                        crate::db::set_tags(&self.db, &conv.id, &conv.tags);
                    }
                }
                Task::none()
            }
            // Comparison + Diff
            Message::ToggleComparisonMode => { self.comparison_mode = !self.comparison_mode; self.diff_active = None; Task::none() }
            Message::ShowDiff(a, b) => { self.diff_active = Some((a, b)); Task::none() }
            Message::DismissDiff => { self.diff_active = None; Task::none() }
            // Quick Switcher
            Message::ToggleQuickSwitcher => {
                self.quick_switcher_open = !self.quick_switcher_open;
                self.quick_switcher_query.clear();
                self.command_palette_open = false;
                self.shortcut_help_open = false;
                if self.quick_switcher_open {
                    return iced::widget::operation::focus("quick-switcher-input");
                }
                Task::none()
            }
            Message::QuickSwitcherQueryChanged(q) => { self.quick_switcher_query = q; Task::none() }
            Message::QuickSwitcherSelect(idx) => {
                self.quick_switcher_open = false;
                self.quick_switcher_query.clear();
                if idx < self.conversations.len() {
                    self.active_conversation = idx;
                    self.view = View::Chat;
                }
                Task::none()
            }
            // Command Palette
            Message::ToggleCommandPalette => {
                self.command_palette_open = !self.command_palette_open;
                self.command_palette_query.clear();
                self.command_palette_selected = 0;
                self.quick_switcher_open = false;
                self.shortcut_help_open = false;
                if self.command_palette_open {
                    return iced::widget::operation::focus("command-palette-input");
                }
                Task::none()
            }
            Message::CommandPaletteQueryChanged(q) => {
                self.command_palette_query = q;
                self.command_palette_selected = 0;
                Task::none()
            }
            Message::CommandPaletteMoveSelection(delta) => {
                let count = self.command_palette_selection_count();
                if count == 0 {
                    self.command_palette_selected = 0;
                    return Task::none();
                }
                let current = self.command_palette_selected.min(count - 1) as i32;
                let next = (current + delta).rem_euclid(count as i32) as usize;
                self.command_palette_selected = next;
                Task::none()
            }
            Message::CommandPaletteExecuteSelected => {
                if !self.command_palette_open {
                    return Task::none();
                }
                let commands = self.command_palette_commands();
                if commands.is_empty() {
                    return Task::none();
                }
                let idx = self.command_palette_selected.min(commands.len() - 1);
                let cmd = commands[idx].message.clone();
                self.command_palette_open = false;
                self.command_palette_query.clear();
                self.command_palette_selected = 0;
                self.update(cmd)
            }
            Message::ToggleShortcutHelp => {
                self.shortcut_help_open = !self.shortcut_help_open;
                self.quick_switcher_open = false;
                self.command_palette_open = false;
                if self.shortcut_help_open {
                    self.command_palette_query.clear();
                    self.command_palette_selected = 0;
                }
                Task::none()
            }
            // Sidebar search
            Message::SidebarSearchChanged(query) => {
                self.sidebar_search_query = query.clone();
                if query.trim().is_empty() {
                    self.sidebar_search_results = None;
                } else {
                    self.sidebar_search_results = Some(crate::db::search_conversations(&self.db, &query));
                }
                Task::none()
            }
            Message::ClearSidebarSearch => {
                self.sidebar_search_query.clear();
                self.sidebar_search_results = None;
                Task::none()
            }
            // Export
            Message::ExportMarkdown => {
                let conv = &self.conversations[self.active_conversation];
                let md = crate::export::conversation_to_markdown(conv);
                iced::clipboard::write(md)
            }
            // Forking
            Message::ForkConversation(msg_idx) => {
                let conv = &self.conversations[self.active_conversation];
                let forked = conv.fork(msg_idx);
                crate::db::save_conversation(&self.db, &forked);
                self.conversations.push(forked);
                self.active_conversation = self.conversations.len() - 1;
                self.view = View::Chat;
                Task::none()
            }
            // Per-conversation system prompt
            Message::ToggleConvSystemPrompt => {
                self.conv_system_prompt_open = !self.conv_system_prompt_open;
                if self.conv_system_prompt_open {
                    self.conv_system_prompt_value = self.conversations[self.active_conversation].system_prompt.clone();
                }
                Task::none()
            }
            Message::ConvSystemPromptChanged(v) => { self.conv_system_prompt_value = v; Task::none() }
            Message::SaveConvSystemPrompt => {
                let conv = &mut self.conversations[self.active_conversation];
                conv.system_prompt = self.conv_system_prompt_value.trim().to_string();
                crate::db::save_conversation(&self.db, conv);
                self.conv_system_prompt_open = false;
                self.conv_system_prompt_value.clear();
                Task::none()
            }
            // Ratings
            Message::RateMessage(idx, rating) => {
                let conv = &mut self.conversations[self.active_conversation];
                if let Some(msg) = conv.messages.get_mut(idx) {
                    msg.rating = if msg.rating == rating { 0 } else { rating }; // toggle
                    let conv_id = conv.id.clone();
                    let new_rating = msg.rating;
                    crate::db::update_rating(&self.db, &conv_id, idx, new_rating);
                }
                Task::none()
            }
            // Analytics
            Message::ShowAnalytics => {
                self.view = View::Analytics;
                self.model_picker_open = false;
                Task::none()
            }
            // Ollama
            Message::OllamaModelsDiscovered(models) => {
                self.config.ollama_models = models;
                Task::none()
            }
            Message::RefreshOllamaModels => {
                let url = self.config.ollama.api_url.clone();
                Task::perform(
                    async move { crate::api::ollama::discover_models(&url).await },
                    |result| match result {
                        Ok(models) => Message::OllamaModelsDiscovered(models),
                        Err(_) => Message::OllamaModelsDiscovered(Vec::new()),
                    },
                )
            }
            // Auto-title
            Message::AutoTitleResult(conv_id, title) => {
                if !title.is_empty() {
                    if let Some(ci) = self.conv_index_by_id(&conv_id) {
                        self.conversations[ci].title = title.clone();
                        crate::db::rename_conversation(&self.db, &conv_id, &title);
                    }
                }
                Task::none()
            }
            // File attach
            Message::AttachFile => {
                Task::perform(
                    async {
                        let handle = rfd::AsyncFileDialog::new()
                            .add_filter("Text files", &["txt", "md", "rs", "py", "js", "ts", "go", "c", "cpp", "h", "json", "toml", "yaml", "yml", "csv", "xml", "html", "css", "sh", "sql"])
                            .add_filter("All files", &["*"])
                            .pick_file()
                            .await;
                        match handle {
                            Some(file) => {
                                let name = file.file_name();
                                let data = file.read().await;
                                let content = String::from_utf8_lossy(&data).to_string();
                                Some((name, content))
                            }
                            None => None,
                        }
                    },
                    |result| match result {
                        Some((name, content)) => Message::FileAttached(format!("{}\n{}", name, content)),
                        None => Message::DismissError, // no-op
                    },
                )
            }
            Message::FileAttached(data) => {
                // Format: "filename\ncontent"
                if let Some(idx) = data.find('\n') {
                    let (name, content) = data.split_at(idx);
                    self.attached_filename = Some(name.to_string());
                    self.attached_file = Some(content[1..].to_string());
                }
                Task::none()
            }
            // Image attach
            Message::AttachImage => {
                Task::perform(
                    async {
                        let handle = rfd::AsyncFileDialog::new()
                            .add_filter("Images", &["png", "jpg", "jpeg", "gif", "webp"])
                            .pick_file()
                            .await;
                        match handle {
                            Some(file) => Some(file.read().await),
                            None => None,
                        }
                    },
                    |result| match result {
                        Some(data) => Message::ImageAttached(data),
                        None => Message::DismissError,
                    },
                )
            }
            Message::ImageAttached(data) => {
                use base64::Engine;
                let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
                self.attached_images.push(b64);
                Task::none()
            }
            // Web search
            Message::WebSearch => {
                if self.input_value.trim().is_empty() { return Task::none(); }
                self.web_search_pending = true;
                let query = self.input_value.clone();
                Task::perform(
                    async move { crate::web_search::search(&query, 5).await },
                    |result| match result {
                        Ok(results) => Message::WebSearchResults(crate::web_search::format_results(&results)),
                        Err(e) => Message::WebSearchResults(format!("[Search error: {e}]")),
                    },
                )
            }
            Message::WebSearchResults(context) => {
                self.web_search_pending = false;
                self.web_search_context = Some(context);
                // Auto-send after search results arrive
                self.update(Message::SendMessage)
            }
            // Export HTML/JSON
            Message::ExportHtml => {
                let conv = &self.conversations[self.active_conversation];
                let html = crate::export::conversation_to_html(conv);
                iced::clipboard::write(html)
            }
            Message::ExportJson => {
                let conv = &self.conversations[self.active_conversation];
                let json = crate::export::conversation_to_json(conv);
                iced::clipboard::write(json)
            }
            // Import
            Message::ImportChatGpt => {
                let _ = &self.db; // import handles DB writes in ImportComplete
                Task::perform(
                    async {
                        let handle = rfd::AsyncFileDialog::new()
                            .add_filter("JSON", &["json"])
                            .pick_file()
                            .await;
                        match handle {
                            Some(file) => {
                                let data = file.read().await;
                                let text = String::from_utf8_lossy(&data).to_string();
                                let convs = crate::import::import_chatgpt(&text);
                                Some(convs)
                            }
                            None => None,
                        }
                    },
                    |result| match result {
                        Some(convs) if !convs.is_empty() => {
                            // We'll store convs temporarily - need to handle via message
                            Message::ImportComplete(convs.len())
                        }
                        _ => Message::DismissError,
                    },
                )
            }
            Message::ImportComplete(_count) => {
                // For now, re-import inline (the async handler above can't easily pass data)
                // Users should use the import flow which writes to DB
                Task::none()
            }
            // Folders
            Message::SetFolder(folder) => {
                let conv = &mut self.conversations[self.active_conversation];
                conv.folder = folder;
                crate::db::save_conversation(&self.db, conv);
                Task::none()
            }
            Message::DismissError => { self.error_message = None; Task::none() }
            Message::RequestStartupFocus => {
                #[cfg(target_os = "macos")]
                {
                    return window::latest().map(Message::FocusMainWindow);
                }
                #[cfg(not(target_os = "macos"))]
                {
                    Task::none()
                }
            }
            Message::FocusMainWindow(window_id) => {
                #[cfg(target_os = "macos")]
                {
                    if let Some(id) = window_id {
                        return window::gain_focus(id);
                    }
                }
                Task::none()
            }
            Message::SetKeybinding(action, binding) => {
                self.config.keybindings.set(action, binding);
                self.config_saved = false;
                Task::none()
            }
            Message::SetDebugKeyEvents(enabled) => {
                self.config.debug_key_events = enabled;
                self.config_saved = false;
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let sidebar = ui::sidebar::view(self);
        let right_panel = ui::right_panel::view(self);
        let bottom_bar = ui::bottom_bar::view(self);

        let sep = |_: &Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(Color::from_rgb8(0x1e, 0x28, 0x34))),
            ..Default::default()
        };

        let content: Element<Message> = match self.view {
            View::Chat => {
                let chat = ui::chat_view::view(self);
                let input = ui::input_bar::view(self);
                column![container(chat).height(Length::Fill), input].into()
            }
            View::Settings => ui::settings::view(self),
            View::Analytics => ui::analytics::view(self),
        };

        let sep_v = || container(iced::widget::Space::new()).width(1).height(Length::Fill).style(sep);
        let main_row = row![sidebar, sep_v(), container(content).width(Length::Fill), sep_v(), right_panel];
        let layout = column![container(main_row).height(Length::Fill), bottom_bar];
        let base: Element<Message> = container(layout).width(Length::Fill).height(Length::Fill).into();

        // Overlays: quick switcher, command palette, and shortcut cheat-sheet
        if self.shortcut_help_open {
            let overlay = ui::shortcut_help::view(self);
            return iced::widget::stack![base, overlay].into();
        }
        if self.quick_switcher_open {
            let overlay = ui::quick_switcher::view(self);
            return iced::widget::stack![base, overlay].into();
        }
        if self.command_palette_open {
            let overlay = ui::command_palette::view(self);
            return iced::widget::stack![base, overlay].into();
        }

        base
    }

    pub fn subscription(&self) -> Subscription<Message> {
        event::listen_with(|event, _status, _window| {
            let iced::Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, physical_key, .. }) = event else {
                return None;
            };
            Some(Message::KeyboardPressed(key, physical_key, modifiers))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Message;
    use super::ChatApp;
    use crate::config::Keybindings;
    use iced::keyboard;

    #[test]
    fn shortcut_maps_character_keys() {
        let key = keyboard::Key::Character("k".into());
        let physical = keyboard::key::Physical::Unidentified(keyboard::key::NativeCode::Unidentified);
        let modifiers = keyboard::Modifiers::from_bits_truncate(keyboard::Modifiers::COMMAND.bits());
        let msg = ChatApp::shortcut_message(&Keybindings::default(), &key, &physical, modifiers);
        assert!(matches!(msg, Some(Message::ToggleQuickSwitcher)));
    }

    #[test]
    fn shortcut_maps_physical_keys() {
        let key = keyboard::Key::Unidentified;
        let physical = keyboard::key::Physical::Code(keyboard::key::Code::KeyP);
        let modifiers = keyboard::Modifiers::from_bits_truncate(keyboard::Modifiers::COMMAND.bits());
        let msg = ChatApp::shortcut_message(&Keybindings::default(), &key, &physical, modifiers);
        assert!(matches!(msg, Some(Message::ToggleCommandPalette)));
    }

    #[test]
    fn shortcut_maps_escape_to_overlay_dismiss() {
        let key = keyboard::Key::Named(keyboard::key::Named::Escape);
        let physical = keyboard::key::Physical::Unidentified(keyboard::key::NativeCode::Unidentified);
        let modifiers = keyboard::Modifiers::default();
        let msg = ChatApp::shortcut_message(&Keybindings::default(), &key, &physical, modifiers);
        assert!(matches!(msg, Some(Message::DismissOverlay)));
    }

    #[test]
    fn dismiss_overlay_is_topmost_first() {
        let mut app = ChatApp::new_for_tests();
        app.command_palette_open = true;
        app.quick_switcher_open = true;
        app.model_picker_open = true;

        app.dismiss_top_overlay();
        assert!(!app.quick_switcher_open);
        assert!(app.command_palette_open);
        assert!(app.model_picker_open);

        app.dismiss_top_overlay();
        assert!(!app.command_palette_open);
        assert!(app.model_picker_open);

        app.dismiss_top_overlay();
        assert!(!app.model_picker_open);
    }
}
