use rusqlite::Connection;
use tauri::State;

use crate::db::DbState;
use crate::models::{Attachment, Message, SendMessageRequest};

// ---------------------------------------------------------------------------
// Core logic functions (testable without Tauri runtime)
// ---------------------------------------------------------------------------

pub fn db_list_messages(conn: &Connection, topic_id: &str) -> Result<Vec<Message>, String> {
    let mut msg_stmt = conn
        .prepare(
            "SELECT id, topic_id, sender_type, sender_bot_id, content, created_at
             FROM messages
             WHERE topic_id = ?1
             ORDER BY created_at ASC",
        )
        .map_err(|e| e.to_string())?;

    let messages = msg_stmt
        .query_map(rusqlite::params![topic_id], |row| {
            Ok(Message {
                id: row.get(0)?,
                topic_id: row.get(1)?,
                sender_type: row.get(2)?,
                sender_bot_id: row.get(3)?,
                content: row.get(4)?,
                created_at: row.get(5)?,
                attachments: Vec::new(),
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<Message>, _>>()
        .map_err(|e| e.to_string())?;

    // For each message, query its attachments
    let mut att_stmt = conn
        .prepare(
            "SELECT id, message_id, file_name, file_path, file_type, mime_type, created_at
             FROM attachments
             WHERE message_id = ?1
             ORDER BY created_at ASC",
        )
        .map_err(|e| e.to_string())?;

    let mut result = Vec::with_capacity(messages.len());
    for mut msg in messages {
        let attachments = att_stmt
            .query_map(rusqlite::params![msg.id], |row| {
                Ok(Attachment {
                    id: row.get(0)?,
                    message_id: row.get(1)?,
                    file_name: row.get(2)?,
                    file_path: row.get(3)?,
                    file_type: row.get(4)?,
                    mime_type: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<Attachment>, _>>()
            .map_err(|e| e.to_string())?;

        msg.attachments = attachments;
        result.push(msg);
    }

    Ok(result)
}

pub fn db_send_human_message(
    conn: &Connection,
    req: SendMessageRequest,
) -> Result<Message, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let created_at = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO messages (id, topic_id, sender_type, sender_bot_id, content, created_at)
         VALUES (?1, ?2, 'human', NULL, ?3, ?4)",
        rusqlite::params![id, req.topic_id, req.content, created_at],
    )
    .map_err(|e| e.to_string())?;

    // Update topic updated_at
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE topics SET updated_at = ?1 WHERE id = ?2",
        rusqlite::params![now, req.topic_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(Message {
        id,
        topic_id: req.topic_id,
        sender_type: "human".to_string(),
        sender_bot_id: None,
        content: req.content,
        created_at,
        attachments: Vec::new(),
    })
}

pub fn db_save_bot_message(
    conn: &Connection,
    topic_id: &str,
    bot_id: &str,
    content: &str,
) -> Result<Message, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let created_at = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO messages (id, topic_id, sender_type, sender_bot_id, content, created_at)
         VALUES (?1, ?2, 'bot', ?3, ?4, ?5)",
        rusqlite::params![id, topic_id, bot_id, content, created_at],
    )
    .map_err(|e| e.to_string())?;

    // Update topic updated_at
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE topics SET updated_at = ?1 WHERE id = ?2",
        rusqlite::params![now, topic_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(Message {
        id,
        topic_id: topic_id.to_string(),
        sender_type: "bot".to_string(),
        sender_bot_id: Some(bot_id.to_string()),
        content: content.to_string(),
        created_at,
        attachments: Vec::new(),
    })
}

// ---------------------------------------------------------------------------
// Tauri commands (thin wrappers)
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn list_messages(db: State<DbState>, topic_id: String) -> Result<Vec<Message>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    db_list_messages(&conn, &topic_id)
}

#[tauri::command]
pub fn send_human_message(db: State<DbState>, req: SendMessageRequest) -> Result<Message, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    db_send_human_message(&conn, req)
}

#[tauri::command]
pub fn save_bot_message(
    db: State<DbState>,
    topic_id: String,
    bot_id: String,
    content: String,
) -> Result<Message, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    db_save_bot_message(&conn, &topic_id, &bot_id, &content)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::SendMessageRequest;
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        crate::db::schema::run_migrations(&conn).unwrap();
        conn
    }

    fn create_test_topic(conn: &Connection) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO topics (id, title, created_at, updated_at) VALUES (?1, 'Test', datetime('now'), datetime('now'))",
            [&id],
        )
        .unwrap();
        id
    }

    fn create_test_bot(conn: &Connection) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO bots (id, name, base_url, model, created_at) VALUES (?1, 'TestBot', 'http://localhost', 'test-model', datetime('now'))",
            [&id],
        )
        .unwrap();
        id
    }

    /// UT-MSG-01: Save human message -> stored with sender_type="human", sender_bot_id=NULL
    #[test]
    fn test_send_human_message() {
        let conn = setup_test_db();
        let topic_id = create_test_topic(&conn);

        let msg = db_send_human_message(
            &conn,
            SendMessageRequest {
                topic_id: topic_id.clone(),
                content: "Hello from human".to_string(),
            },
        )
        .unwrap();

        assert!(!msg.id.is_empty());
        assert_eq!(msg.topic_id, topic_id);
        assert_eq!(msg.sender_type, "human");
        assert!(msg.sender_bot_id.is_none());
        assert_eq!(msg.content, "Hello from human");
        assert!(!msg.created_at.is_empty());
        assert!(msg.attachments.is_empty());

        // Verify in DB
        let sender_bot_id: Option<String> = conn
            .query_row(
                "SELECT sender_bot_id FROM messages WHERE id = ?1",
                rusqlite::params![msg.id],
                |row| row.get(0),
            )
            .unwrap();
        assert!(sender_bot_id.is_none());
    }

    /// UT-MSG-02: Save bot message -> stored with sender_type="bot", correct sender_bot_id
    #[test]
    fn test_save_bot_message() {
        let conn = setup_test_db();
        let topic_id = create_test_topic(&conn);
        let bot_id = create_test_bot(&conn);

        let msg = db_save_bot_message(&conn, &topic_id, &bot_id, "Hello from bot").unwrap();

        assert!(!msg.id.is_empty());
        assert_eq!(msg.topic_id, topic_id);
        assert_eq!(msg.sender_type, "bot");
        assert_eq!(msg.sender_bot_id, Some(bot_id.clone()));
        assert_eq!(msg.content, "Hello from bot");
        assert!(!msg.created_at.is_empty());

        // Verify in DB
        let db_bot_id: Option<String> = conn
            .query_row(
                "SELECT sender_bot_id FROM messages WHERE id = ?1",
                rusqlite::params![msg.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(db_bot_id, Some(bot_id));
    }

    /// UT-MSG-03: List messages for topic, ordered by created_at ASC
    #[test]
    fn test_list_messages_ordered() {
        let conn = setup_test_db();
        let topic_id = create_test_topic(&conn);

        // Insert messages with explicit timestamps
        conn.execute(
            "INSERT INTO messages (id, topic_id, sender_type, content, created_at) VALUES ('m1', ?1, 'human', 'First', '2024-01-01T00:00:00Z')",
            rusqlite::params![topic_id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO messages (id, topic_id, sender_type, content, created_at) VALUES ('m2', ?1, 'human', 'Third', '2024-03-01T00:00:00Z')",
            rusqlite::params![topic_id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO messages (id, topic_id, sender_type, content, created_at) VALUES ('m3', ?1, 'human', 'Second', '2024-02-01T00:00:00Z')",
            rusqlite::params![topic_id],
        )
        .unwrap();

        let messages = db_list_messages(&conn, &topic_id).unwrap();
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0].content, "First");
        assert_eq!(messages[1].content, "Second");
        assert_eq!(messages[2].content, "Third");
    }

    /// UT-MSG-04: List messages returns empty for new topic
    #[test]
    fn test_list_messages_empty() {
        let conn = setup_test_db();
        let topic_id = create_test_topic(&conn);

        let messages = db_list_messages(&conn, &topic_id).unwrap();
        assert!(messages.is_empty());
    }

    /// UT-MSG-05: Messages isolated between topics (topic A messages not in topic B)
    #[test]
    fn test_messages_isolated_between_topics() {
        let conn = setup_test_db();
        let topic_a = create_test_topic(&conn);
        let topic_b = create_test_topic(&conn);

        db_send_human_message(
            &conn,
            SendMessageRequest {
                topic_id: topic_a.clone(),
                content: "Message in A".to_string(),
            },
        )
        .unwrap();

        db_send_human_message(
            &conn,
            SendMessageRequest {
                topic_id: topic_b.clone(),
                content: "Message in B".to_string(),
            },
        )
        .unwrap();

        let messages_a = db_list_messages(&conn, &topic_a).unwrap();
        assert_eq!(messages_a.len(), 1);
        assert_eq!(messages_a[0].content, "Message in A");

        let messages_b = db_list_messages(&conn, &topic_b).unwrap();
        assert_eq!(messages_b.len(), 1);
        assert_eq!(messages_b[0].content, "Message in B");
    }

    /// UT-MSG-06: Save message updates topic updated_at
    #[test]
    fn test_save_message_updates_topic_updated_at() {
        let conn = setup_test_db();

        // Create topic with a known old timestamp
        let topic_id = uuid::Uuid::new_v4().to_string();
        let old_time = "2020-01-01T00:00:00+00:00";
        conn.execute(
            "INSERT INTO topics (id, title, created_at, updated_at) VALUES (?1, 'Test', ?2, ?2)",
            rusqlite::params![topic_id, old_time],
        )
        .unwrap();

        // Verify initial updated_at
        let before: String = conn
            .query_row(
                "SELECT updated_at FROM topics WHERE id = ?1",
                rusqlite::params![topic_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(before, old_time);

        // Send a message
        db_send_human_message(
            &conn,
            SendMessageRequest {
                topic_id: topic_id.clone(),
                content: "Update me".to_string(),
            },
        )
        .unwrap();

        // Verify updated_at changed
        let after: String = conn
            .query_row(
                "SELECT updated_at FROM topics WHERE id = ?1",
                rusqlite::params![topic_id],
                |row| row.get(0),
            )
            .unwrap();
        assert!(after > before, "updated_at should have been updated");
    }
}
