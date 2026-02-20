use serde::{Deserialize, Serialize};

use crate::model::{Provider, ProviderConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub active_provider: Provider,
    pub openai: ProviderConfig,
    pub anthropic: ProviderConfig,
    #[serde(default = "ProviderConfig::default_ollama")]
    pub ollama: ProviderConfig,
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
}

fn default_temperature() -> String { "0.7".to_string() }
fn default_max_tokens() -> String { "4096".to_string() }

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            active_provider: Provider::OpenAI,
            openai: ProviderConfig::default_openai(),
            anthropic: ProviderConfig::default_anthropic(),
            ollama: ProviderConfig::default_ollama(),
            system_prompt: String::new(),
            temperature: "0.7".to_string(),
            max_tokens: "4096".to_string(),
            selected_model: None,
            ollama_models: Vec::new(),
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
            Provider::Ollama => &self.ollama,
        }
    }

    pub fn active_provider_config_mut(&mut self) -> &mut ProviderConfig {
        match self.active_provider {
            Provider::OpenAI => &mut self.openai,
            Provider::Anthropic => &mut self.anthropic,
            Provider::Ollama => &mut self.ollama,
        }
    }

    /// Build a ProviderConfig for a specific model id, using the stored API keys/URLs.
    pub fn provider_config_for_model(&self, model: &str) -> ProviderConfig {
        // Check if it's an Ollama model
        if self.ollama_models.contains(&model.to_string()) {
            return ProviderConfig {
                provider: Provider::Ollama,
                api_url: self.ollama.api_url.clone(),
                api_key: String::new(),
                model: model.to_string(),
            };
        }
        let is_anthropic = model.contains("claude")
            || model.contains("anthropic")
            || model.contains("haiku")
            || model.contains("sonnet")
            || model.contains("opus");
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

    /// Hardcoded list of cloud (display_name, model_id).
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

    /// All models: cloud + discovered Ollama models.
    pub fn all_models(&self) -> Vec<(String, String)> {
        let mut out: Vec<(String, String)> = Self::available_models()
            .iter()
            .map(|(d, id)| (d.to_string(), id.to_string()))
            .collect();
        for m in &self.ollama_models {
            out.push((m.clone(), m.clone()));
        }
        out
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
