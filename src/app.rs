use iced::widget::{column, container, row};
use iced::{event, keyboard, window, Element, Length, Subscription, Task, Theme};
use rusqlite::Connection;
use std::collections::{HashMap, HashSet};
use std::time::Instant;

use crate::config::AppConfig;
use crate::model::{Conversation, Provider};
use crate::theme::ThemeName;
use crate::ui;

pub type StreamId = usize;

#[derive(Debug, Clone)]
pub enum View {
    Chat,
    Settings,
    Analytics,
    Diagnostics,
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
    pub(crate) db: Connection,
    // Shared HTTP client
    pub http_client: reqwest::Client,
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
    // Diagnostics
    pub startup_focus_attempts: u32,
    pub startup_focus_successes: u32,
    pub diagnostics_last_run: Option<String>,
    pub last_shortcut_event: Option<String>,
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
    ShowDiagnostics,
    RunDiagnostics,
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
    ImportComplete(Vec<Conversation>), // imported conversations to save
    // Folders
    #[allow(dead_code)]
    SetFolder(Option<String>),
    // Misc
    DismissError,
    RequestStartupFocus,
    FocusMainWindow(Option<window::Id>),
    SetKeybinding(crate::shortcuts::ShortcutAction, String),
    ResetKeybindings,
    SetDebugKeyEvents(bool),
    SetTheme(ThemeName),
    KeyboardPressed(keyboard::Key, keyboard::key::Physical, keyboard::Modifiers),
}

impl ChatApp {
    fn from_parts(config: AppConfig, db: Connection, conversations: Vec<Conversation>) -> Self {
        crate::theme::set_theme(config.theme);

        let conversations = if conversations.is_empty() {
            let c = Conversation::new();
            let _ = crate::db::save_conversation(&db, &c);
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
            http_client: crate::api::new_shared_client(),
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
            startup_focus_attempts: 0,
            startup_focus_successes: 0,
            diagnostics_last_run: None,
            last_shortcut_event: None,
        }
    }

    fn active_conv(&self) -> Option<&Conversation> {
        self.conversations.get(self.active_conversation)
    }

    pub(crate) fn handle_db_result(error_message: &mut Option<String>, result: Result<(), String>) {
        if let Err(e) = result {
            log::error!("DB error: {e}");
            *error_message = Some(e);
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

    #[allow(dead_code)]
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
        let Some(conv) = self.active_conv() else { return false };
        self.active_streams.values().any(|s| s.conversation_id == conv.id)
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
        Theme::custom("Stoa".to_string(), self.config.theme.iced_palette())
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
                self.last_shortcut_event = Some(format!(
                    "key={key:?} physical={physical_key:?} cmd={} ctrl={} shift={} alt={} mapped={mapped:?}",
                    modifiers.command(),
                    modifiers.control(),
                    modifiers.shift(),
                    modifiers.alt(),
                ));
                // Only log key events with modifier keys to avoid capturing sensitive typed text
                if self.config.debug_key_events && (modifiers.command() || modifiers.control() || modifiers.alt()) {
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
            Message::SendMessage => self.handle_send_message(),
            Message::SendToModels(model_ids) => self.handle_send_to_models(model_ids),
            Message::SendToAll => self.handle_send_to_all(),
            Message::ToggleModelSelection(model_id) => {
                if self.selected_models.contains(&model_id) { self.selected_models.remove(&model_id); }
                else { self.selected_models.insert(model_id); }
                Task::none()
            }
            Message::StreamToken(id, token) => self.handle_stream_token(id, token),
            Message::StreamComplete(id) => self.handle_stream_complete(id),
            Message::StreamError(id, err) => self.handle_stream_error(id, err),
            Message::StopStreaming => self.handle_stop_streaming(),
            Message::StopStream(id) => self.handle_stop_stream(id),
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
                Self::handle_db_result(&mut self.error_message,crate::db::save_conversation(&self.db, &conv));
                self.conversations.push(conv);
                self.active_conversation = self.conversations.len() - 1;
                self.view = View::Chat;
                self.model_picker_open = false;
                Task::none()
            }
            Message::DeleteConversation(idx) => {
                if self.conversations.len() <= 1 { return Task::none(); }
                let conv = &self.conversations[idx];
                Self::handle_db_result(&mut self.error_message,crate::db::delete_conversation(&self.db, &conv.id));
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
                        let title = self.conversations[idx].title.clone();
                        Self::handle_db_result(&mut self.error_message,crate::db::rename_conversation(&self.db, &id, &title));
                    }
                }
                self.rename_value.clear();
                Task::none()
            }
            Message::DismissOverlay => {
                self.dismiss_top_overlay();
                Task::none()
            }
            Message::RetryMessage => self.handle_retry_message(),
            Message::DeleteMessage(idx) => {
                let Some(conv) = self.conversations.get_mut(self.active_conversation) else { return Task::none() };
                if idx < conv.messages.len() {
                    conv.messages.remove(idx);
                    Self::handle_db_result(&mut self.error_message,crate::db::save_conversation(&self.db, conv));
                }
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
            Message::ReviewWith(model_id) => self.handle_review_with(model_id),
            Message::AnalyzeConversation(idx) => {
                if idx < self.conversations.len() { self.analyze_source_conversation = Some(idx); }
                Task::none()
            }
            Message::AnalyzeWith(model_id) => self.handle_analyze_with(model_id),
            Message::DismissAnalyzePicker => { self.analyze_source_conversation = None; Task::none() }
            // Tags + Pins
            Message::TogglePin(idx) => {
                if idx < self.conversations.len() {
                    self.conversations[idx].pinned = !self.conversations[idx].pinned;
                    let id = self.conversations[idx].id.clone();
                    let pinned = self.conversations[idx].pinned;
                    Self::handle_db_result(&mut self.error_message,crate::db::toggle_pin(&self.db, &id, pinned));
                    // Re-sort: pinned first, then by original order
                    let active_id = self.conversations.get(self.active_conversation).map(|c| c.id.clone()).unwrap_or_default();
                    self.conversations.sort_by(|a, b| b.pinned.cmp(&a.pinned));
                    self.active_conversation = self.conversations.iter().position(|c| c.id == active_id).unwrap_or(0);
                }
                Task::none()
            }
            Message::RemoveTag(tag) => {
                let Some(conv) = self.conversations.get_mut(self.active_conversation) else { return Task::none() };
                conv.tags.retain(|t| t != &tag);
                let id = conv.id.clone();
                let tags = conv.tags.clone();
                Self::handle_db_result(&mut self.error_message,crate::db::set_tags(&self.db, &id, &tags));
                Task::none()
            }
            Message::ToggleTagInput => { self.tag_input_open = !self.tag_input_open; self.tag_input_value.clear(); Task::none() }
            Message::TagInputChanged(v) => { self.tag_input_value = v; Task::none() }
            Message::SubmitTag => {
                let tag = self.tag_input_value.trim().to_string();
                self.tag_input_value.clear();
                self.tag_input_open = false;
                if !tag.is_empty() {
                    let Some(conv) = self.conversations.get_mut(self.active_conversation) else { return Task::none() };
                    if !conv.tags.contains(&tag) {
                        conv.tags.push(tag);
                        let id = conv.id.clone();
                        let tags = conv.tags.clone();
                        Self::handle_db_result(&mut self.error_message,crate::db::set_tags(&self.db, &id, &tags));
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
                let trimmed = query.trim();
                if trimmed.is_empty() {
                    self.sidebar_search_results = None;
                } else if trimmed.len() >= 2 {
                    self.sidebar_search_results = Some(crate::db::search_conversations(&self.db, trimmed));
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
                let Some(conv) = self.conversations.get(self.active_conversation) else { return Task::none() };
                let md = crate::export::conversation_to_markdown(conv);
                iced::clipboard::write(md)
            }
            // Forking
            Message::ForkConversation(msg_idx) => {
                let Some(conv) = self.conversations.get(self.active_conversation) else { return Task::none() };
                let forked = conv.fork(msg_idx);
                Self::handle_db_result(&mut self.error_message,crate::db::save_conversation(&self.db, &forked));
                self.conversations.push(forked);
                self.active_conversation = self.conversations.len() - 1;
                self.view = View::Chat;
                Task::none()
            }
            // Per-conversation system prompt
            Message::ToggleConvSystemPrompt => {
                self.conv_system_prompt_open = !self.conv_system_prompt_open;
                if self.conv_system_prompt_open {
                    if let Some(conv) = self.conversations.get(self.active_conversation) {
                        self.conv_system_prompt_value = conv.system_prompt.clone();
                    }
                }
                Task::none()
            }
            Message::ConvSystemPromptChanged(v) => { self.conv_system_prompt_value = v; Task::none() }
            Message::SaveConvSystemPrompt => {
                let Some(conv) = self.conversations.get_mut(self.active_conversation) else { return Task::none() };
                conv.system_prompt = self.conv_system_prompt_value.trim().to_string();
                Self::handle_db_result(&mut self.error_message,crate::db::save_conversation(&self.db, conv));
                self.conv_system_prompt_open = false;
                self.conv_system_prompt_value.clear();
                Task::none()
            }
            // Ratings
            Message::RateMessage(idx, rating) => {
                let Some(conv) = self.conversations.get_mut(self.active_conversation) else { return Task::none() };
                if let Some(msg) = conv.messages.get_mut(idx) {
                    msg.rating = if msg.rating == rating { 0 } else { rating }; // toggle
                    let conv_id = conv.id.clone();
                    let new_rating = msg.rating;
                    Self::handle_db_result(&mut self.error_message,crate::db::update_rating(&self.db, &conv_id, idx, new_rating));
                }
                Task::none()
            }
            // Analytics
            Message::ShowAnalytics => {
                self.view = View::Analytics;
                self.model_picker_open = false;
                Task::none()
            }
            Message::ShowDiagnostics => {
                self.view = View::Diagnostics;
                self.model_picker_open = false;
                Task::none()
            }
            Message::RunDiagnostics => {
                let ts = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                self.diagnostics_last_run = Some(format!("unix:{ts}"));
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
                        Self::handle_db_result(&mut self.error_message,crate::db::rename_conversation(&self.db, &conv_id, &title));
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
                let Some(conv) = self.conversations.get(self.active_conversation) else { return Task::none() };
                let html = crate::export::conversation_to_html(conv);
                iced::clipboard::write(html)
            }
            Message::ExportJson => {
                let Some(conv) = self.conversations.get(self.active_conversation) else { return Task::none() };
                let json = crate::export::conversation_to_json(conv);
                iced::clipboard::write(json)
            }
            // Import
            Message::ImportChatGpt => {
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
                                if convs.is_empty() { None } else { Some(convs) }
                            }
                            None => None,
                        }
                    },
                    |result| match result {
                        Some(convs) => Message::ImportComplete(convs),
                        None => Message::DismissError,
                    },
                )
            }
            Message::ImportComplete(convs) => {
                let count = convs.len();
                for conv in convs {
                    Self::handle_db_result(&mut self.error_message,crate::db::save_conversation(&self.db, &conv));
                    self.conversations.push(conv);
                }
                if count > 0 {
                    self.active_conversation = self.conversations.len() - 1;
                    self.view = View::Chat;
                }
                Task::none()
            }
            // Folders
            Message::SetFolder(folder) => {
                let Some(conv) = self.conversations.get_mut(self.active_conversation) else { return Task::none() };
                conv.folder = folder;
                Self::handle_db_result(&mut self.error_message,crate::db::save_conversation(&self.db, conv));
                Task::none()
            }
            Message::DismissError => { self.error_message = None; Task::none() }
            Message::RequestStartupFocus => {
                self.startup_focus_attempts = self.startup_focus_attempts.saturating_add(1);
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
                        self.startup_focus_successes = self.startup_focus_successes.saturating_add(1);
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
            Message::ResetKeybindings => {
                self.config.keybindings = crate::config::Keybindings::default();
                self.config_saved = false;
                Task::none()
            }
            Message::SetDebugKeyEvents(enabled) => {
                self.config.debug_key_events = enabled;
                self.config_saved = false;
                Task::none()
            }
            Message::SetTheme(name) => {
                self.config.theme = name;
                crate::theme::set_theme(name);
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
            background: Some(iced::Background::Color(crate::theme::SEPARATOR())),
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
            View::Diagnostics => ui::diagnostics::view(self),
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

    #[test]
    fn startup_focus_request_records_attempt() {
        let mut app = ChatApp::new_for_tests();
        let _ = app.update(Message::RequestStartupFocus);
        assert!(app.startup_focus_attempts >= 1);
    }

    #[test]
    fn reset_keybindings_restores_defaults() {
        let mut app = ChatApp::new_for_tests();
        app.config.keybindings.new_conversation = "Ctrl+Shift+N".to_string();
        let _ = app.update(Message::ResetKeybindings);
        assert_eq!(
            app.config.keybindings.new_conversation,
            crate::shortcuts::default_binding(crate::shortcuts::ShortcutAction::NewConversation)
        );
    }
}
