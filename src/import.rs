use crate::model::{ChatMessage, Conversation, Role};

/// Import conversations from ChatGPT's export format (conversations.json).
pub fn import_chatgpt(data: &str) -> Vec<Conversation> {
    let parsed: Result<Vec<serde_json::Value>, _> = serde_json::from_str(data);
    let items = match parsed {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let mut conversations = Vec::new();
    for item in &items {
        let title = item["title"].as_str().unwrap_or("Imported Chat").to_string();
        let mapping = match item.get("mapping") {
            Some(m) => m,
            None => continue,
        };

        let mut messages = Vec::new();

        // ChatGPT export uses a mapping of node_id -> message node
        // We need to traverse the tree to get messages in order
        let mut nodes: Vec<(&str, &serde_json::Value)> = mapping.as_object()
            .map(|obj| obj.iter().map(|(k, v)| (k.as_str(), v)).collect())
            .unwrap_or_default();

        // Sort by create_time to get chronological order
        nodes.sort_by(|a, b| {
            let time_a = a.1["message"]["create_time"].as_f64().unwrap_or(0.0);
            let time_b = b.1["message"]["create_time"].as_f64().unwrap_or(0.0);
            time_a.partial_cmp(&time_b).unwrap_or(std::cmp::Ordering::Equal)
        });

        for (_node_id, node) in &nodes {
            let msg = match node.get("message") {
                Some(m) if !m.is_null() => m,
                _ => continue,
            };

            let role_str = msg["author"]["role"].as_str().unwrap_or("");
            let role = match role_str {
                "user" => Role::User,
                "assistant" => Role::Assistant,
                _ => continue,
            };

            // Content can be in parts array or content.text
            let content = if let Some(parts) = msg["content"]["parts"].as_array() {
                parts.iter()
                    .filter_map(|p| p.as_str())
                    .collect::<Vec<_>>()
                    .join("\n")
            } else if let Some(text) = msg["content"]["text"].as_str() {
                text.to_string()
            } else {
                continue;
            };

            if content.trim().is_empty() {
                continue;
            }

            let model = msg["metadata"]["model_slug"].as_str().map(|s| s.to_string());

            messages.push(ChatMessage {
                role,
                content,
                streaming: false,
                model,
                token_count: None,
                rating: 0,
                latency_ms: None,
                images: Vec::new(),
            });
        }

        if !messages.is_empty() {
            let mut conv = Conversation::new();
            conv.title = title;
            conv.messages = messages;
            conv.tags = vec!["imported".to_string()];
            conversations.push(conv);
        }
    }

    conversations
}
