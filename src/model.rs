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
    #[serde(default)]
    pub token_count: Option<u32>,
    /// -1 = thumbs down, 0 = no rating, 1 = thumbs up
    #[serde(default)]
    pub rating: i8,
    /// Time to first token in milliseconds
    #[serde(default)]
    pub latency_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub messages: Vec<ChatMessage>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub pinned: bool,
    #[serde(default)]
    pub system_prompt: String,
    #[serde(default)]
    pub forked_from: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Provider {
    OpenAI,
    Anthropic,
    Ollama,
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
            tags: Vec::new(),
            pinned: false,
            system_prompt: String::new(),
            forked_from: None,
        }
    }

    /// Fork this conversation up to (and including) message at `up_to_index`.
    pub fn fork(&self, up_to_index: usize) -> Self {
        let messages: Vec<ChatMessage> = self.messages.iter()
            .take(up_to_index + 1)
            .filter(|m| !m.streaming)
            .cloned()
            .collect();
        Self {
            id: Uuid::new_v4().to_string(),
            title: format!("Fork of {}", self.title),
            messages,
            tags: self.tags.clone(),
            pinned: false,
            system_prompt: self.system_prompt.clone(),
            forked_from: Some(self.id.clone()),
        }
    }

    pub fn add_user_message(&mut self, content: &str, target_model: Option<String>) {
        self.messages.push(ChatMessage {
            role: Role::User,
            content: content.to_string(),
            streaming: false,
            model: target_model,
            token_count: None,
            rating: 0,
            latency_ms: None,
        });
        if self.title == "New Chat" && !content.trim().is_empty() {
            self.title = content.chars().take(30).collect();
        }
    }

    pub fn push_streaming_assistant(&mut self, model: Option<String>) -> usize {
        let idx = self.messages.len();
        self.messages.push(ChatMessage {
            role: Role::Assistant,
            content: String::new(),
            streaming: true,
            model,
            token_count: None,
            rating: 0,
            latency_ms: None,
        });
        idx
    }

    pub fn update_streaming_at(&mut self, index: usize, content: &str) {
        if let Some(msg) = self.messages.get_mut(index) {
            if msg.role == Role::Assistant && msg.streaming {
                msg.content = content.to_string();
            }
        }
    }

    pub fn finalize_at(&mut self, index: usize, content: &str) {
        if let Some(msg) = self.messages.get_mut(index) {
            if msg.role == Role::Assistant {
                msg.content = content.to_string();
                msg.streaming = false;
            }
        }
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

    pub fn default_ollama() -> Self {
        Self {
            provider: Provider::Ollama,
            api_url: "http://localhost:11434/v1/chat/completions".to_string(),
            api_key: String::new(),
            model: "llama3.2".to_string(),
        }
    }
}
