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
    #[serde(default)]
    pub streaming: bool,
    #[serde(default)]
    pub model: Option<String>,
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

    pub fn add_user_message(&mut self, content: &str, target_model: Option<String>) {
        self.messages.push(ChatMessage {
            role: Role::User,
            content: content.to_string(),
            streaming: false,
            model: target_model,
        });
        if self.title == "New Chat" && !content.trim().is_empty() {
            self.title = content.chars().take(30).collect();
        }
    }

    pub fn update_assistant_streaming(&mut self, content: &str, model: Option<String>) {
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
            model,
        });
    }

    pub fn finalize_assistant_message(&mut self, content: &str, model: Option<String>) {
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
            model,
        });
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
