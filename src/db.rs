use rusqlite::{Connection, params};

use crate::model::{ChatMessage, Conversation, Role};

/// Open an in-memory database for testing.
pub fn open_in_memory() -> Connection {
    let conn = Connection::open_in_memory().expect("failed to open in-memory database");
    init_schema(&conn);
    conn
}

fn init_schema(conn: &Connection) {
    conn.execute_batch(
        "PRAGMA foreign_keys = ON;

         CREATE TABLE IF NOT EXISTS conversations (
             id TEXT PRIMARY KEY,
             title TEXT NOT NULL,
             updated_at TEXT DEFAULT (datetime('now'))
         );

         CREATE TABLE IF NOT EXISTS messages (
             id INTEGER PRIMARY KEY AUTOINCREMENT,
             conversation_id TEXT NOT NULL,
             role TEXT NOT NULL CHECK (role IN ('user', 'assistant')),
             content TEXT NOT NULL,
             FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
         );"
    ).expect("failed to create tables");

    // Migrations (idempotent)
    conn.execute("ALTER TABLE conversations ADD COLUMN updated_at TEXT", []).ok();
    conn.execute("ALTER TABLE messages ADD COLUMN model TEXT", []).ok();
    conn.execute("ALTER TABLE conversations ADD COLUMN tags TEXT DEFAULT ''", []).ok();
    conn.execute("ALTER TABLE conversations ADD COLUMN pinned INTEGER DEFAULT 0", []).ok();
    conn.execute("ALTER TABLE messages ADD COLUMN token_count INTEGER", []).ok();
    conn.execute("ALTER TABLE conversations ADD COLUMN system_prompt TEXT DEFAULT ''", []).ok();
    conn.execute("ALTER TABLE conversations ADD COLUMN forked_from TEXT", []).ok();
    conn.execute("ALTER TABLE messages ADD COLUMN rating INTEGER DEFAULT 0", []).ok();
    conn.execute("ALTER TABLE messages ADD COLUMN latency_ms INTEGER", []).ok();
    conn.execute("ALTER TABLE conversations ADD COLUMN folder TEXT", []).ok();
}

pub fn open() -> Connection {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("stoa");
    std::fs::create_dir_all(&dir).ok();
    let path = dir.join("chat.db");
    let conn = Connection::open(&path).expect("failed to open database");

    conn.execute_batch("PRAGMA journal_mode = WAL;").ok();
    init_schema(&conn);
    conn.execute("UPDATE conversations SET updated_at = datetime('now') WHERE updated_at IS NULL", []).ok();

    migrate_from_json(&conn);
    conn
}

pub fn load_all(conn: &Connection) -> Vec<Conversation> {
    let mut stmt = conn
        .prepare("SELECT id, title, COALESCE(tags, ''), COALESCE(pinned, 0), COALESCE(system_prompt, ''), forked_from, folder FROM conversations ORDER BY pinned DESC, updated_at DESC, rowid DESC")
        .expect("failed to prepare query");

    let conv_rows: Vec<(String, String, String, i32, String, Option<String>, Option<String>)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?)))
        .expect("failed to query conversations")
        .filter_map(|r| r.ok())
        .collect();

    let mut msg_stmt = conn
        .prepare("SELECT role, content, model, token_count, COALESCE(rating, 0), latency_ms FROM messages WHERE conversation_id = ?1 ORDER BY id")
        .expect("failed to prepare message query");

    conv_rows
        .into_iter()
        .map(|(id, title, tags_str, pinned, system_prompt, forked_from, folder)| {
            let messages: Vec<ChatMessage> = msg_stmt
                .query_map(params![id], |row| {
                    let role_str: String = row.get(0)?;
                    let content: String = row.get(1)?;
                    let model: Option<String> = row.get(2)?;
                    let token_count: Option<u32> = row.get(3)?;
                    let rating: i8 = row.get::<_, i32>(4)? as i8;
                    let latency_ms: Option<u64> = row.get(5)?;
                    Ok(ChatMessage {
                        role: if role_str == "user" { Role::User } else { Role::Assistant },
                        content,
                        streaming: false,
                        model,
                        token_count,
                        rating,
                        latency_ms,
                        images: Vec::new(),
                    })
                })
                .expect("failed to query messages")
                .filter_map(|r| r.ok())
                .collect();

            let tags = if tags_str.is_empty() {
                Vec::new()
            } else {
                tags_str.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
            };

            Conversation { id, title, messages, tags, pinned: pinned != 0, system_prompt, forked_from, folder }
        })
        .collect()
}

pub fn save_conversation(conn: &Connection, conv: &Conversation) {
    conn.execute(
        "INSERT OR REPLACE INTO conversations (id, title, tags, pinned, system_prompt, forked_from, folder, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, datetime('now'))",
        params![conv.id, conv.title, conv.tags.join(","), conv.pinned as i32, conv.system_prompt, conv.forked_from, conv.folder],
    ).ok();

    conn.execute(
        "DELETE FROM messages WHERE conversation_id = ?1",
        params![conv.id],
    ).ok();

    let mut stmt = conn
        .prepare("INSERT INTO messages (conversation_id, role, content, model, token_count, rating, latency_ms) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)")
        .expect("failed to prepare insert");

    for msg in &conv.messages {
        if msg.streaming {
            continue;
        }
        let role_str = match msg.role {
            Role::User => "user",
            Role::Assistant => "assistant",
        };
        stmt.execute(params![conv.id, role_str, msg.content, msg.model, msg.token_count, msg.rating as i32, msg.latency_ms]).ok();
    }
}

pub fn update_rating(conn: &Connection, conv_id: &str, msg_index: usize, rating: i8) {
    // Get the message's rowid by position
    let mut stmt = conn.prepare(
        "SELECT id FROM messages WHERE conversation_id = ?1 ORDER BY id LIMIT 1 OFFSET ?2"
    ).expect("rating query failed");
    if let Ok(msg_id) = stmt.query_row(params![conv_id, msg_index], |row| row.get::<_, i64>(0)) {
        conn.execute("UPDATE messages SET rating = ?1 WHERE id = ?2", params![rating as i32, msg_id]).ok();
    }
}

pub fn delete_conversation(conn: &Connection, id: &str) {
    conn.execute("DELETE FROM conversations WHERE id = ?1", params![id]).ok();
}

pub fn rename_conversation(conn: &Connection, id: &str, title: &str) {
    conn.execute(
        "UPDATE conversations SET title = ?1 WHERE id = ?2",
        params![title, id],
    ).ok();
}

pub fn toggle_pin(conn: &Connection, id: &str, pinned: bool) {
    conn.execute(
        "UPDATE conversations SET pinned = ?1 WHERE id = ?2",
        params![pinned as i32, id],
    ).ok();
}

pub fn set_tags(conn: &Connection, id: &str, tags: &[String]) {
    conn.execute(
        "UPDATE conversations SET tags = ?1 WHERE id = ?2",
        params![tags.join(","), id],
    ).ok();
}

pub fn search_conversations(conn: &Connection, query: &str) -> Vec<String> {
    let pattern = format!("%{query}%");
    let mut stmt = conn.prepare(
        "SELECT DISTINCT c.id FROM conversations c
         LEFT JOIN messages m ON m.conversation_id = c.id
         WHERE c.title LIKE ?1 OR m.content LIKE ?1
         ORDER BY c.pinned DESC, c.updated_at DESC"
    ).expect("search query failed");

    stmt.query_map(params![pattern], |row| row.get(0))
        .expect("search failed")
        .filter_map(|r| r.ok())
        .collect()
}

fn migrate_from_json(conn: &Connection) {
    let json_dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("stoa")
        .join("conversations");

    if !json_dir.exists() {
        return;
    }

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM conversations", [], |row| row.get(0))
        .unwrap_or(0);

    if count > 0 {
        return;
    }

    if let Ok(entries) = std::fs::read_dir(&json_dir) {
        for entry in entries.flatten() {
            if entry.path().extension().is_some_and(|e| e == "json") {
                if let Ok(data) = std::fs::read_to_string(entry.path()) {
                    if let Ok(conv) = serde_json::from_str::<Conversation>(&data) {
                        save_conversation(conn, &conv);
                    }
                }
            }
        }
    }

    eprintln!("[stoa] migrated JSON conversations to SQLite");
}
