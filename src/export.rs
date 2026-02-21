use crate::model::{Conversation, Role};

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

pub fn conversation_to_markdown(conv: &Conversation) -> String {
    let mut md = format!("# {}\n\n", conv.title);
    if !conv.tags.is_empty() {
        md.push_str(&format!("**Tags:** {}\n\n", conv.tags.join(", ")));
    }
    for msg in &conv.messages {
        if msg.streaming { continue; }
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

pub fn conversation_to_json(conv: &Conversation) -> String {
    serde_json::to_string_pretty(conv).unwrap_or_else(|_| "{}".to_string())
}

pub fn conversation_to_html(conv: &Conversation) -> String {
    let title = escape_html(&conv.title);
    let mut html = format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<title>{title}</title>
<style>
body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; max-width: 800px; margin: 0 auto; padding: 20px; background: #111922; color: #e8e0d0; }}
h1 {{ color: #c9a84c; }}
.tags {{ color: #8a909a; font-size: 14px; margin-bottom: 20px; }}
.message {{ margin-bottom: 24px; padding: 16px; border-radius: 8px; }}
.user {{ background: #1e2836; text-align: right; }}
.assistant {{ background: #161e2a; }}
.role {{ font-size: 12px; color: #8a909a; margin-bottom: 8px; }}
.content {{ white-space: pre-wrap; line-height: 1.6; }}
.meta {{ font-size: 11px; color: #505a66; margin-top: 8px; }}
hr {{ border: none; border-top: 1px solid #1e2834; margin: 16px 0; }}
</style>
</head>
<body>
<h1>{title}</h1>
"#);

    if !conv.tags.is_empty() {
        let tags_escaped: Vec<String> = conv.tags.iter().map(|t| escape_html(t)).collect();
        html.push_str(&format!("<p class=\"tags\">Tags: {}</p>\n", tags_escaped.join(", ")));
    }

    for msg in &conv.messages {
        if msg.streaming { continue; }
        let (class, label) = match msg.role {
            Role::User => ("user", "You".to_string()),
            Role::Assistant => ("assistant", match &msg.model {
                Some(m) => format!("Assistant ({})", escape_html(m)),
                None => "Assistant".to_string(),
            }),
        };
        let escaped = escape_html(&msg.content);
        let mut meta_parts = Vec::new();
        if let Some(tokens) = msg.token_count {
            meta_parts.push(format!("{tokens} tokens"));
        }
        if let Some(lat) = msg.latency_ms {
            meta_parts.push(format!("{lat} ms"));
        }
        let meta = if meta_parts.is_empty() { String::new() } else {
            format!("<div class=\"meta\">{}</div>", meta_parts.join(" · "))
        };
        html.push_str(&format!(
            "<div class=\"message {class}\"><div class=\"role\">{label}</div><div class=\"content\">{escaped}</div>{meta}</div>\n"
        ));
    }

    html.push_str("</body>\n</html>\n");
    html
}
