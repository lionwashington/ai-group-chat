use rusqlite::Connection;

pub fn seed_default_bots(conn: &Connection) -> Result<(), rusqlite::Error> {
    let count: i32 = conn.query_row("SELECT COUNT(*) FROM bots", [], |row| row.get(0))?;
    if count > 0 {
        return Ok(());
    }

    conn.execute(
        "INSERT INTO bots (id, name, avatar_color, base_url, api_key, model, system_prompt, supports_vision)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            uuid::Uuid::new_v4().to_string(),
            "Claude Sonnet 4",
            "#8b5cf6",
            "http://127.0.0.1:8317/v1",
            "your-api-key-1",
            "claude-sonnet-4-20250514",
            "You are Claude Sonnet 4, a helpful AI assistant by Anthropic.",
            1,
        ],
    )?;

    conn.execute(
        "INSERT INTO bots (id, name, avatar_color, base_url, api_key, model, system_prompt, supports_vision)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            uuid::Uuid::new_v4().to_string(),
            "Claude Opus 4.6",
            "#6366f1",
            "http://127.0.0.1:8317/v1",
            "your-api-key-1",
            "claude-opus-4-6",
            "You are Claude Opus 4.6, the most capable AI assistant by Anthropic.",
            1,
        ],
    )?;

    conn.execute(
        "INSERT INTO bots (id, name, avatar_color, base_url, api_key, model, system_prompt, supports_vision)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            uuid::Uuid::new_v4().to_string(),
            "Gemini 2.5 Pro",
            "#10b981",
            "http://127.0.0.1:8317/v1",
            "your-api-key-1",
            "gemini-2.5-pro",
            "You are Gemini 2.5 Pro, a helpful AI assistant by Google.",
            1,
        ],
    )?;

    conn.execute(
        "INSERT INTO bots (id, name, avatar_color, base_url, api_key, model, system_prompt, supports_vision)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            uuid::Uuid::new_v4().to_string(),
            "Gemini 3.1 Pro",
            "#f97316",
            "http://127.0.0.1:8317/v1",
            "your-api-key-1",
            "gemini-3.1-pro-preview",
            "You are Gemini 3.1 Pro, a helpful AI assistant by Google.",
            1,
        ],
    )?;

    Ok(())
}

pub fn run_migrations(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS bots (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            avatar_color TEXT NOT NULL DEFAULT '#6366f1',
            base_url TEXT NOT NULL,
            api_key TEXT NOT NULL DEFAULT '',
            model TEXT NOT NULL,
            system_prompt TEXT NOT NULL DEFAULT '',
            supports_vision INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS topics (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS topic_bots (
            topic_id TEXT NOT NULL REFERENCES topics(id) ON DELETE CASCADE,
            bot_id TEXT NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
            PRIMARY KEY (topic_id, bot_id)
        );

        CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            topic_id TEXT NOT NULL REFERENCES topics(id) ON DELETE CASCADE,
            sender_type TEXT NOT NULL CHECK(sender_type IN ('human', 'bot')),
            sender_bot_id TEXT REFERENCES bots(id) ON DELETE SET NULL,
            content TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS attachments (
            id TEXT PRIMARY KEY,
            message_id TEXT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
            file_name TEXT NOT NULL,
            file_path TEXT NOT NULL,
            file_type TEXT NOT NULL CHECK(file_type IN ('image', 'file')),
            mime_type TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE INDEX IF NOT EXISTS idx_messages_topic ON messages(topic_id, created_at);
        CREATE INDEX IF NOT EXISTS idx_attachments_message ON attachments(message_id);
    ",
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    /// UT-DB-01: Run migrations on fresh DB
    /// Verifies that all 5 tables are created successfully.
    #[test]
    fn test_migrations_fresh_db() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        run_migrations(&conn).unwrap();
        // Verify tables exist by querying sqlite_master
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('bots','topics','topic_bots','messages','attachments')",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 5);
    }

    /// UT-DB-02: Run migrations twice (idempotent)
    /// Verifies that running migrations a second time does not error.
    #[test]
    fn test_migrations_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        run_migrations(&conn).unwrap();
        run_migrations(&conn).unwrap(); // Second run should not error
    }

    /// UT-DB-03: Foreign key constraints enabled
    /// Verifies that inserting a message with a nonexistent topic_id fails.
    #[test]
    fn test_foreign_key_constraints() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        run_migrations(&conn).unwrap();
        // Try inserting a message with invalid topic_id
        let result = conn.execute(
            "INSERT INTO messages (id, topic_id, sender_type, content) VALUES ('m1', 'nonexistent', 'human', 'hello')",
            [],
        );
        assert!(result.is_err());
    }
}
