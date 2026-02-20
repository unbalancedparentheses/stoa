use crate::model::{Conversation, Role};

pub fn conversation_to_markdown(conv: &Conversation) -> String {
    let mut md = format!("# {}\n\n", conv.title);

    if !conv.tags.is_empty() {
        md.push_str(&format!("**Tags:** {}\n\n", conv.tags.join(", ")));
    }

    for msg in &conv.messages {
        if msg.streaming {
            continue;
        }
        let label = match msg.role {
            Role::User => "**You**".to_string(),
            Role::Assistant => match &msg.model {
                Some(m) => format!("**Assistant ({m})**"),
                None => "**Assistant**".to_string(),
            },
        };
        md.push_str(&format!("{label}\n\n{}\n\n---\n\n", msg.content));
    }

    md
}
