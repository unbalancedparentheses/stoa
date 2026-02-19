use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Role {
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
    pub streaming: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub messages: Vec<ChatMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Provider {
    OpenAI,
    Anthropic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub provider: Provider,
    pub api_url: String,
    pub api_key: String,
    pub model: String,
}

impl Conversation {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            title: "New Chat".to_string(),
            messages: Vec::new(),
        }
    }

    pub fn add_user_message(&mut self, content: &str) {
        self.messages.push(ChatMessage {
            role: Role::User,
            content: content.to_string(),
            streaming: false,
        });
        if self.title == "New Chat" && !content.trim().is_empty() {
            self.title = content.chars().take(30).collect();
        }
    }

    pub fn update_assistant_streaming(&mut self, content: &str) {
        if let Some(last) = self.messages.last_mut() {
            if last.role == Role::Assistant && last.streaming {
                last.content = content.to_string();
                return;
            }
        }
        self.messages.push(ChatMessage {
            role: Role::Assistant,
            content: content.to_string(),
            streaming: true,
        });
    }

    pub fn finalize_assistant_message(&mut self, content: &str) {
        if let Some(last) = self.messages.last_mut() {
            if last.role == Role::Assistant && last.streaming {
                last.content = content.to_string();
                last.streaming = false;
                return;
            }
        }
        self.messages.push(ChatMessage {
            role: Role::Assistant,
            content: content.to_string(),
            streaming: false,
        });
    }

    fn data_dir() -> std::path::PathBuf {
        let dir = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("rust-chat")
            .join("conversations");
        std::fs::create_dir_all(&dir).ok();
        dir
    }

    pub fn save(&self) {
        let path = Self::data_dir().join(format!("{}.json", self.id));
        if let Ok(json) = serde_json::to_string_pretty(self) {
            std::fs::write(path, json).ok();
        }
    }

    pub fn delete(&self) {
        let path = Self::data_dir().join(format!("{}.json", self.id));
        std::fs::remove_file(path).ok();
    }


    pub fn load_all() -> Vec<Self> {
        let dir = Self::data_dir();
        let mut convos = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                if entry.path().extension().is_some_and(|e| e == "json") {
                    if let Ok(data) = std::fs::read_to_string(entry.path()) {
                        if let Ok(conv) = serde_json::from_str::<Conversation>(&data) {
                            convos.push(conv);
                        }
                    }
                }
            }
        }
        convos
    }
}

impl ProviderConfig {
    pub fn default_openai() -> Self {
        Self {
            provider: Provider::OpenAI,
            api_url: "https://api.openai.com/v1/chat/completions".to_string(),
            api_key: String::new(),
            model: "gpt-4.1".to_string(),
        }
    }

    pub fn default_anthropic() -> Self {
        Self {
            provider: Provider::Anthropic,
            api_url: "https://api.anthropic.com/v1/messages".to_string(),
            api_key: String::new(),
            model: "claude-sonnet-4-20250514".to_string(),
        }
    }
}
