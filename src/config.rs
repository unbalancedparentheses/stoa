use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::model::{Provider, ProviderConfig};
use crate::shortcuts::{self, ShortcutAction};

pub const CONFIG_SCHEMA_VERSION: u32 = 2;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    pub active_provider: Provider,
    pub openai: ProviderConfig,
    pub anthropic: ProviderConfig,
    #[serde(default = "ProviderConfig::default_ollama")]
    pub ollama: ProviderConfig,
    #[serde(default = "ProviderConfig::default_openrouter")]
    pub openrouter: ProviderConfig,
    #[serde(default)]
    pub system_prompt: String,
    #[serde(default = "default_temperature")]
    pub temperature: String,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: String,
    #[serde(default)]
    pub selected_model: Option<String>,
    #[serde(default)]
    pub ollama_models: Vec<String>,
    #[serde(default)]
    pub keybindings: Keybindings,
    #[serde(default)]
    pub debug_key_events: bool,
}

fn default_temperature() -> String { "0.7".to_string() }
fn default_max_tokens() -> String { "4096".to_string() }
fn default_schema_version() -> u32 { CONFIG_SCHEMA_VERSION }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keybindings {
    #[serde(default = "default_send_to_all")]
    pub send_to_all: String,
    #[serde(default = "default_new_conversation")]
    pub new_conversation: String,
    #[serde(default = "default_show_settings")]
    pub show_settings: String,
    #[serde(default = "default_quick_switcher")]
    pub quick_switcher: String,
    #[serde(default = "default_command_palette")]
    pub command_palette: String,
    #[serde(default = "default_export_markdown")]
    pub export_markdown: String,
    #[serde(default = "default_toggle_shortcut_help")]
    pub toggle_shortcut_help: String,
}

fn default_send_to_all() -> String { shortcuts::default_binding(ShortcutAction::SendToAll).to_string() }
fn default_new_conversation() -> String { shortcuts::default_binding(ShortcutAction::NewConversation).to_string() }
fn default_show_settings() -> String { shortcuts::default_binding(ShortcutAction::ShowSettings).to_string() }
fn default_quick_switcher() -> String { shortcuts::default_binding(ShortcutAction::QuickSwitcher).to_string() }
fn default_command_palette() -> String { shortcuts::default_binding(ShortcutAction::CommandPalette).to_string() }
fn default_export_markdown() -> String { shortcuts::default_binding(ShortcutAction::ExportMarkdown).to_string() }
fn default_toggle_shortcut_help() -> String { shortcuts::default_binding(ShortcutAction::ToggleShortcutHelp).to_string() }

impl Default for Keybindings {
    fn default() -> Self {
        Self {
            send_to_all: default_send_to_all(),
            new_conversation: default_new_conversation(),
            show_settings: default_show_settings(),
            quick_switcher: default_quick_switcher(),
            command_palette: default_command_palette(),
            export_markdown: default_export_markdown(),
            toggle_shortcut_help: default_toggle_shortcut_help(),
        }
    }
}

impl Keybindings {
    pub fn get(&self, action: ShortcutAction) -> &str {
        match action {
            ShortcutAction::SendToAll => &self.send_to_all,
            ShortcutAction::NewConversation => &self.new_conversation,
            ShortcutAction::ShowSettings => &self.show_settings,
            ShortcutAction::QuickSwitcher => &self.quick_switcher,
            ShortcutAction::CommandPalette => &self.command_palette,
            ShortcutAction::ExportMarkdown => &self.export_markdown,
            ShortcutAction::ToggleShortcutHelp => &self.toggle_shortcut_help,
        }
    }

    pub fn set(&mut self, action: ShortcutAction, binding: String) {
        match action {
            ShortcutAction::SendToAll => self.send_to_all = binding,
            ShortcutAction::NewConversation => self.new_conversation = binding,
            ShortcutAction::ShowSettings => self.show_settings = binding,
            ShortcutAction::QuickSwitcher => self.quick_switcher = binding,
            ShortcutAction::CommandPalette => self.command_palette = binding,
            ShortcutAction::ExportMarkdown => self.export_markdown = binding,
            ShortcutAction::ToggleShortcutHelp => self.toggle_shortcut_help = binding,
        }
    }

    pub fn conflicts(&self) -> Vec<(String, Vec<ShortcutAction>)> {
        let mut by_binding: HashMap<String, Vec<ShortcutAction>> = HashMap::new();
        for spec in shortcuts::specs() {
            let key = self.get(spec.action).trim().to_string();
            if key.is_empty() {
                continue;
            }
            by_binding.entry(key).or_default().push(spec.action);
        }

        let mut out: Vec<(String, Vec<ShortcutAction>)> = by_binding
            .into_iter()
            .filter_map(|(binding, actions)| {
                if actions.len() > 1 {
                    Some((binding, actions))
                } else {
                    None
                }
            })
            .collect();
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            schema_version: CONFIG_SCHEMA_VERSION,
            active_provider: Provider::OpenAI,
            openai: ProviderConfig::default_openai(),
            anthropic: ProviderConfig::default_anthropic(),
            ollama: ProviderConfig::default_ollama(),
            openrouter: ProviderConfig::default_openrouter(),
            system_prompt: String::new(),
            temperature: "0.7".to_string(),
            max_tokens: "4096".to_string(),
            selected_model: None,
            ollama_models: Vec::new(),
            keybindings: Keybindings::default(),
            debug_key_events: false,
        }
    }
}

impl AppConfig {
    fn config_path() -> std::path::PathBuf {
        let dir = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("stoa");
        std::fs::create_dir_all(&dir).ok();
        dir.join("config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(data) = std::fs::read_to_string(&path) {
                if let Ok(mut config) = serde_json::from_str::<AppConfig>(&data) {
                    config.migrate();
                    return config;
                }
                if let Ok(mut legacy) = serde_json::from_str::<serde_json::Value>(&data) {
                    let schema_version = legacy["schema_version"].as_u64().unwrap_or(1);
                    if schema_version < CONFIG_SCHEMA_VERSION as u64 {
                        legacy["schema_version"] = serde_json::json!(CONFIG_SCHEMA_VERSION);
                    }
                    if let Ok(mut config) = serde_json::from_value::<AppConfig>(legacy) {
                        config.migrate();
                        return config;
                    }
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        let path = Self::config_path();
        let mut copy = self.clone();
        copy.schema_version = CONFIG_SCHEMA_VERSION;
        if let Ok(json) = serde_json::to_string_pretty(&copy) {
            std::fs::write(path, json).ok();
        }
    }

    pub fn migrate(&mut self) {
        if self.schema_version == 0 {
            self.schema_version = 1;
        }
        // v1 -> v2: keybindings/debug fields were added; serde defaults already fill missing values.
        if self.schema_version < 2 {
            self.schema_version = 2;
        }
        if self.schema_version > CONFIG_SCHEMA_VERSION {
            self.schema_version = CONFIG_SCHEMA_VERSION;
        }
    }

    pub fn active_provider_config(&self) -> &ProviderConfig {
        match self.active_provider {
            Provider::OpenAI => &self.openai,
            Provider::Anthropic => &self.anthropic,
            Provider::Ollama => &self.ollama,
            Provider::OpenRouter => &self.openrouter,
        }
    }

    pub fn active_provider_config_mut(&mut self) -> &mut ProviderConfig {
        match self.active_provider {
            Provider::OpenAI => &mut self.openai,
            Provider::Anthropic => &mut self.anthropic,
            Provider::Ollama => &mut self.ollama,
            Provider::OpenRouter => &mut self.openrouter,
        }
    }

    pub fn provider_config_for_model(&self, model: &str) -> ProviderConfig {
        // Ollama models (discovered)
        if self.ollama_models.contains(&model.to_string()) {
            return ProviderConfig {
                provider: Provider::Ollama,
                api_url: self.ollama.api_url.clone(),
                api_key: String::new(),
                model: model.to_string(),
            };
        }
        // OpenRouter models (contain /)
        if model.contains('/') {
            return ProviderConfig {
                provider: Provider::OpenRouter,
                api_url: self.openrouter.api_url.clone(),
                api_key: self.openrouter.api_key.clone(),
                model: model.to_string(),
            };
        }
        // Anthropic models
        let is_anthropic = model.contains("claude") || model.contains("anthropic")
            || model.contains("haiku") || model.contains("sonnet") || model.contains("opus");
        if is_anthropic {
            ProviderConfig {
                provider: Provider::Anthropic,
                api_url: self.anthropic.api_url.clone(),
                api_key: self.anthropic.api_key.clone(),
                model: model.to_string(),
            }
        } else {
            ProviderConfig {
                provider: Provider::OpenAI,
                api_url: self.openai.api_url.clone(),
                api_key: self.openai.api_key.clone(),
                model: model.to_string(),
            }
        }
    }

    /// Hardcoded cloud models (direct API).
    pub fn available_models() -> Vec<(&'static str, &'static str)> {
        vec![
            ("GPT-4.1", "gpt-4.1"),
            ("GPT-5", "gpt-5"),
            ("o3", "o3"),
            ("o4-mini", "o4-mini"),
            ("Claude Opus", "claude-opus-4-20250514"),
            ("Claude Sonnet", "claude-sonnet-4-20250514"),
            ("Claude Haiku", "claude-haiku-4-5-20251001"),
        ]
    }

    /// OpenRouter models (accessed via OpenRouter API).
    pub fn openrouter_models() -> Vec<(&'static str, &'static str)> {
        vec![
            ("Gemini 2.5 Flash", "google/gemini-2.5-flash-preview"),
            ("Gemini 2.5 Pro", "google/gemini-2.5-pro-preview"),
            ("Llama 4 Maverick", "meta-llama/llama-4-maverick"),
            ("Llama 4 Scout", "meta-llama/llama-4-scout"),
            ("Mistral Large", "mistralai/mistral-large-2411"),
            ("DeepSeek R1", "deepseek/deepseek-r1"),
            ("DeepSeek V3", "deepseek/deepseek-chat"),
            ("Qwen3 235B", "qwen/qwen3-235b-a22b"),
        ]
    }

    /// All models: direct API + OpenRouter + Ollama.
    pub fn all_models(&self) -> Vec<(String, String)> {
        let mut out: Vec<(String, String)> = Self::available_models()
            .iter()
            .map(|(d, id)| (d.to_string(), id.to_string()))
            .collect();
        // OpenRouter models (only if key is configured)
        if !self.openrouter.api_key.is_empty() {
            for (d, id) in Self::openrouter_models() {
                out.push((d.to_string(), id.to_string()));
            }
        }
        for m in &self.ollama_models {
            out.push((m.clone(), m.clone()));
        }
        out
    }

    pub fn apply_preset(&mut self, preset: &str) {
        match preset {
            "GPT-5" => { self.active_provider = Provider::OpenAI; self.openai.model = "gpt-5".to_string(); }
            "GPT-4.1" => { self.active_provider = Provider::OpenAI; self.openai.model = "gpt-4.1".to_string(); }
            "o3" => { self.active_provider = Provider::OpenAI; self.openai.model = "o3".to_string(); }
            "o4-mini" => { self.active_provider = Provider::OpenAI; self.openai.model = "o4-mini".to_string(); }
            "Opus" => { self.active_provider = Provider::Anthropic; self.anthropic.model = "claude-opus-4-20250514".to_string(); }
            "Sonnet" => { self.active_provider = Provider::Anthropic; self.anthropic.model = "claude-sonnet-4-20250514".to_string(); }
            "Haiku" => { self.active_provider = Provider::Anthropic; self.anthropic.model = "claude-haiku-4-5-20251001".to_string(); }
            _ => {}
        }
    }
}
