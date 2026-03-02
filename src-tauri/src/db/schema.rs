use rusqlite::Connection;

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
