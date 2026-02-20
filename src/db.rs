use rusqlite::{Connection, params};

use crate::model::{ChatMessage, Conversation, Role};

pub fn open() -> Connection {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("rust-chat");
    std::fs::create_dir_all(&dir).ok();
    let path = dir.join("chat.db");
    let conn = Connection::open(&path).expect("failed to open database");

    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA foreign_keys = ON;

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

    // Migration: add updated_at if missing (existing DBs)
    conn.execute(
        "ALTER TABLE conversations ADD COLUMN updated_at TEXT",
        [],
    ).ok();
    // Backfill NULL updated_at so existing rows sort properly
    conn.execute(
        "UPDATE conversations SET updated_at = datetime('now') WHERE updated_at IS NULL",
        [],
    ).ok();

    // Migration: add model column to messages
    conn.execute(
        "ALTER TABLE messages ADD COLUMN model TEXT",
        [],
    ).ok();

    migrate_from_json(&conn);
    conn
}

pub fn load_all(conn: &Connection) -> Vec<Conversation> {
    let mut stmt = conn
        .prepare("SELECT id, title FROM conversations ORDER BY updated_at DESC, rowid DESC")
        .expect("failed to prepare query");

    let conv_rows: Vec<(String, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .expect("failed to query conversations")
        .filter_map(|r| r.ok())
        .collect();

    let mut msg_stmt = conn
        .prepare("SELECT role, content, model FROM messages WHERE conversation_id = ?1 ORDER BY id")
        .expect("failed to prepare message query");

    conv_rows
        .into_iter()
        .map(|(id, title)| {
            let messages: Vec<ChatMessage> = msg_stmt
                .query_map(params![id], |row| {
                    let role_str: String = row.get(0)?;
                    let content: String = row.get(1)?;
                    let model: Option<String> = row.get(2)?;
                    Ok(ChatMessage {
                        role: if role_str == "user" { Role::User } else { Role::Assistant },
                        content,
                        streaming: false,
                        model,
                    })
                })
                .expect("failed to query messages")
                .filter_map(|r| r.ok())
                .collect();

            Conversation { id, title, messages }
        })
        .collect()
}

pub fn save_conversation(conn: &Connection, conv: &Conversation) {
    conn.execute(
        "INSERT OR REPLACE INTO conversations (id, title, updated_at) VALUES (?1, ?2, datetime('now'))",
        params![conv.id, conv.title],
    ).ok();

    conn.execute(
        "DELETE FROM messages WHERE conversation_id = ?1",
        params![conv.id],
    ).ok();

    let mut stmt = conn
        .prepare("INSERT INTO messages (conversation_id, role, content, model) VALUES (?1, ?2, ?3, ?4)")
        .expect("failed to prepare insert");

    for msg in &conv.messages {
        if msg.streaming {
            continue;
        }
        let role_str = match msg.role {
            Role::User => "user",
            Role::Assistant => "assistant",
        };
        stmt.execute(params![conv.id, role_str, msg.content, msg.model]).ok();
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

fn migrate_from_json(conn: &Connection) {
    let json_dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("rust-chat")
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

    eprintln!("[rust-chat] migrated JSON conversations to SQLite");
}
