use serde::{Deserialize, Serialize};

use crate::model::{Provider, ProviderConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub active_provider: Provider,
    pub openai: ProviderConfig,
    pub anthropic: ProviderConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            active_provider: Provider::OpenAI,
            openai: ProviderConfig::default_openai(),
            anthropic: ProviderConfig::default_anthropic(),
        }
    }
}

impl AppConfig {
    fn config_path() -> std::path::PathBuf {
        let dir = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("rust-chat");
        std::fs::create_dir_all(&dir).ok();
        dir.join("config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(data) = std::fs::read_to_string(&path) {
                if let Ok(config) = serde_json::from_str(&data) {
                    return config;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        let path = Self::config_path();
        if let Ok(json) = serde_json::to_string_pretty(self) {
            std::fs::write(path, json).ok();
        }
    }

    pub fn active_provider_config(&self) -> &ProviderConfig {
        match self.active_provider {
            Provider::OpenAI => &self.openai,
            Provider::Anthropic => &self.anthropic,
        }
    }

    pub fn active_provider_config_mut(&mut self) -> &mut ProviderConfig {
        match self.active_provider {
            Provider::OpenAI => &mut self.openai,
            Provider::Anthropic => &mut self.anthropic,
        }
    }

    pub fn apply_preset(&mut self, preset: &str) {
        match preset {
            "GPT-5" => {
                self.active_provider = Provider::OpenAI;
                self.openai.model = "gpt-5".to_string();
            }
            "GPT-4.1" => {
                self.active_provider = Provider::OpenAI;
                self.openai.model = "gpt-4.1".to_string();
            }
            "o3" => {
                self.active_provider = Provider::OpenAI;
                self.openai.model = "o3".to_string();
            }
            "o4-mini" => {
                self.active_provider = Provider::OpenAI;
                self.openai.model = "o4-mini".to_string();
            }
            "Opus" => {
                self.active_provider = Provider::Anthropic;
                self.anthropic.model = "claude-opus-4-20250514".to_string();
            }
            "Sonnet" => {
                self.active_provider = Provider::Anthropic;
                self.anthropic.model = "claude-sonnet-4-20250514".to_string();
            }
            "Haiku" => {
                self.active_provider = Provider::Anthropic;
                self.anthropic.model = "claude-haiku-4-5-20251001".to_string();
            }
            _ => {}
        }
    }
}
