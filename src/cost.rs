use crate::model::{ChatMessage, Role};

/// Estimate tokens from text using chars/4 heuristic.
pub fn estimate_tokens(text: &str) -> u32 {
    (text.len() as f64 / 4.0).ceil() as u32
}

/// (input_price_per_million_tokens, output_price_per_million_tokens)
fn pricing(model: &str) -> Option<(f64, f64)> {
    match model {
        "gpt-4.1" => Some((2.00, 8.00)),
        "gpt-5" => Some((10.00, 30.00)),
        "o3" => Some((10.00, 40.00)),
        "o4-mini" => Some((1.10, 4.40)),
        "claude-opus-4-20250514" => Some((15.00, 75.00)),
        "claude-sonnet-4-20250514" => Some((3.00, 15.00)),
        "claude-haiku-4-5-20251001" => Some((0.80, 4.00)),
        _ => None, // Ollama / unknown = free
    }
}

/// Calculate cost in USD for a single message.
pub fn message_cost(model: &str, role: &Role, token_count: u32) -> f64 {
    if let Some((input_price, output_price)) = pricing(model) {
        let price = match role {
            Role::User => input_price,
            Role::Assistant => output_price,
        };
        (token_count as f64 / 1_000_000.0) * price
    } else {
        0.0
    }
}

/// Calculate total cost for a conversation.
pub fn conversation_cost(messages: &[ChatMessage]) -> f64 {
    messages
        .iter()
        .filter(|m| !m.streaming)
        .map(|m| {
            let tokens = m.token_count.unwrap_or_else(|| estimate_tokens(&m.content));
            let model = m.model.as_deref().unwrap_or("");
            message_cost(model, &m.role, tokens)
        })
        .sum()
}
