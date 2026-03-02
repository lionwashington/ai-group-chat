use rusqlite::Connection;
use tauri::State;

use crate::db::DbState;
use crate::models::{Bot, CreateTopicRequest, Topic, TopicSummary};

// ---------------------------------------------------------------------------
// Core logic functions (testable without Tauri runtime)
// ---------------------------------------------------------------------------

pub fn db_list_topics(conn: &Connection) -> Result<Vec<TopicSummary>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT
                t.id,
                t.title,
                t.updated_at,
                (SELECT COUNT(*) FROM topic_bots tb WHERE tb.topic_id = t.id) AS bot_count,
                (SELECT m.content FROM messages m WHERE m.topic_id = t.id ORDER BY m.created_at DESC LIMIT 1) AS last_message_preview
            FROM topics t
            ORDER BY t.updated_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let topics = stmt
        .query_map([], |row| {
            let bot_count: i64 = row.get(3)?;
            Ok(TopicSummary {
                id: row.get(0)?,
                title: row.get(1)?,
                updated_at: row.get(2)?,
                bot_count: bot_count as usize,
                last_message_preview: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<TopicSummary>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(topics)
}

pub fn db_get_topic(conn: &Connection, id: &str) -> Result<Topic, String> {
    let topic = conn
        .query_row(
            "SELECT id, title, created_at, updated_at FROM topics WHERE id = ?1",
            rusqlite::params![id],
            |row| {
                Ok(Topic {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    created_at: row.get(2)?,
                    updated_at: row.get(3)?,
                    bots: Vec::new(),
                })
            },
        )
        .map_err(|e| format!("Topic not found: {}", e))?;

    // Fetch associated bots
    let mut stmt = conn
        .prepare(
            "SELECT b.id, b.name, b.avatar_color, b.base_url, b.api_key, b.model, b.system_prompt, b.supports_vision, b.created_at
             FROM bots b
             INNER JOIN topic_bots tb ON tb.bot_id = b.id
             WHERE tb.topic_id = ?1
             ORDER BY b.name",
        )
        .map_err(|e| e.to_string())?;

    let bots = stmt
        .query_map(rusqlite::params![id], |row| {
            let supports_vision_int: i32 = row.get(7)?;
            Ok(Bot {
                id: row.get(0)?,
                name: row.get(1)?,
                avatar_color: row.get(2)?,
                base_url: row.get(3)?,
                api_key: row.get(4)?,
                model: row.get(5)?,
                system_prompt: row.get(6)?,
                supports_vision: supports_vision_int != 0,
                created_at: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<Bot>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(Topic { bots, ..topic })
}

pub fn db_create_topic(conn: &Connection, req: CreateTopicRequest) -> Result<Topic, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO topics (id, title, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![id, req.title, now, now],
    )
    .map_err(|e| e.to_string())?;

    // Insert topic_bots entries
    for bot_id in &req.bot_ids {
        conn.execute(
            "INSERT INTO topic_bots (topic_id, bot_id) VALUES (?1, ?2)",
            rusqlite::params![id, bot_id],
        )
        .map_err(|e| e.to_string())?;
    }

    // Return the full topic with bots
    db_get_topic(conn, &id)
}

pub fn db_update_topic_bots(
    conn: &Connection,
    topic_id: &str,
    bot_ids: Vec<String>,
) -> Result<Topic, String> {
    // Delete all existing topic_bots for this topic
    conn.execute(
        "DELETE FROM topic_bots WHERE topic_id = ?1",
        rusqlite::params![topic_id],
    )
    .map_err(|e| e.to_string())?;

    // Insert new ones
    for bot_id in &bot_ids {
        conn.execute(
            "INSERT INTO topic_bots (topic_id, bot_id) VALUES (?1, ?2)",
            rusqlite::params![topic_id, bot_id],
        )
        .map_err(|e| e.to_string())?;
    }

    // Update updated_at
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE topics SET updated_at = ?1 WHERE id = ?2",
        rusqlite::params![now, topic_id],
    )
    .map_err(|e| e.to_string())?;

    db_get_topic(conn, topic_id)
}

pub fn db_delete_topic(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM topics WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tauri commands (thin wrappers)
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn list_topics(db: State<DbState>) -> Result<Vec<TopicSummary>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    db_list_topics(&conn)
}

#[tauri::command]
pub fn get_topic(db: State<DbState>, id: String) -> Result<Topic, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    db_get_topic(&conn, &id)
}

#[tauri::command]
pub fn create_topic(db: State<DbState>, req: CreateTopicRequest) -> Result<Topic, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    db_create_topic(&conn, req)
}

#[tauri::command]
pub fn update_topic_bots(
    db: State<DbState>,
    topic_id: String,
    bot_ids: Vec<String>,
) -> Result<Topic, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    db_update_topic_bots(&conn, &topic_id, bot_ids)
}

#[tauri::command]
pub fn delete_topic(db: State<DbState>, id: String) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    db_delete_topic(&conn, &id)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::CreateBotRequest;
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        crate::db::schema::run_migrations(&conn).unwrap();
        conn
    }

    fn create_test_bot(conn: &Connection, name: &str) -> Bot {
        crate::commands::bot::db_create_bot(
            conn,
            CreateBotRequest {
                name: name.to_string(),
                avatar_color: None,
                base_url: "http://localhost:8080/v1".to_string(),
                api_key: None,
                model: "test-model".to_string(),
                system_prompt: None,
                supports_vision: None,
            },
        )
        .unwrap()
    }

    /// UT-TOPIC-01: Create topic with title and 2 bot_ids -> Topic created, both bots in topic.bots
    #[test]
    fn test_create_topic_with_bots() {
        let conn = setup_test_db();
        let bot1 = create_test_bot(&conn, "Bot1");
        let bot2 = create_test_bot(&conn, "Bot2");

        let topic = db_create_topic(
            &conn,
            CreateTopicRequest {
                title: "Test Topic".to_string(),
                bot_ids: vec![bot1.id.clone(), bot2.id.clone()],
            },
        )
        .unwrap();

        assert!(!topic.id.is_empty());
        assert_eq!(topic.title, "Test Topic");
        assert!(!topic.created_at.is_empty());
        assert!(!topic.updated_at.is_empty());
        assert_eq!(topic.bots.len(), 2);

        let bot_ids: Vec<&str> = topic.bots.iter().map(|b| b.id.as_str()).collect();
        assert!(bot_ids.contains(&bot1.id.as_str()));
        assert!(bot_ids.contains(&bot2.id.as_str()));
    }

    /// UT-TOPIC-02: Create topic with no bots -> Topic created with empty bot list
    #[test]
    fn test_create_topic_no_bots() {
        let conn = setup_test_db();

        let topic = db_create_topic(
            &conn,
            CreateTopicRequest {
                title: "Empty Topic".to_string(),
                bot_ids: vec![],
            },
        )
        .unwrap();

        assert!(!topic.id.is_empty());
        assert_eq!(topic.title, "Empty Topic");
        assert!(topic.bots.is_empty());
    }

    /// UT-TOPIC-03: List topics when empty -> empty vec
    #[test]
    fn test_list_topics_empty() {
        let conn = setup_test_db();
        let topics = db_list_topics(&conn).unwrap();
        assert!(topics.is_empty());
    }

    /// UT-TOPIC-04: List topics ordered by updated_at DESC -> most recent first
    #[test]
    fn test_list_topics_ordered_by_updated_at_desc() {
        let conn = setup_test_db();

        // Create topics with explicit timestamps so ordering is deterministic
        conn.execute(
            "INSERT INTO topics (id, title, created_at, updated_at) VALUES ('t1', 'Old Topic', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO topics (id, title, created_at, updated_at) VALUES ('t2', 'New Topic', '2024-06-01T00:00:00Z', '2024-06-01T00:00:00Z')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO topics (id, title, created_at, updated_at) VALUES ('t3', 'Middle Topic', '2024-03-01T00:00:00Z', '2024-03-01T00:00:00Z')",
            [],
        )
        .unwrap();

        let topics = db_list_topics(&conn).unwrap();
        assert_eq!(topics.len(), 3);
        assert_eq!(topics[0].title, "New Topic");
        assert_eq!(topics[1].title, "Middle Topic");
        assert_eq!(topics[2].title, "Old Topic");
    }

    /// UT-TOPIC-05: Get topic includes associated bots -> correct bots vec
    #[test]
    fn test_get_topic_includes_bots() {
        let conn = setup_test_db();
        let bot1 = create_test_bot(&conn, "AlphaBot");
        let bot2 = create_test_bot(&conn, "BetaBot");

        let created = db_create_topic(
            &conn,
            CreateTopicRequest {
                title: "Topic With Bots".to_string(),
                bot_ids: vec![bot1.id.clone(), bot2.id.clone()],
            },
        )
        .unwrap();

        let topic = db_get_topic(&conn, &created.id).unwrap();
        assert_eq!(topic.title, "Topic With Bots");
        assert_eq!(topic.bots.len(), 2);

        // Bots are ordered by name
        assert_eq!(topic.bots[0].name, "AlphaBot");
        assert_eq!(topic.bots[1].name, "BetaBot");
    }

    /// UT-TOPIC-06: Update topic bots (replace all) -> old bots gone, new ones present
    #[test]
    fn test_update_topic_bots_replaces_all() {
        let conn = setup_test_db();
        let bot1 = create_test_bot(&conn, "Bot1");
        let bot2 = create_test_bot(&conn, "Bot2");
        let bot3 = create_test_bot(&conn, "Bot3");

        let topic = db_create_topic(
            &conn,
            CreateTopicRequest {
                title: "Updatable Topic".to_string(),
                bot_ids: vec![bot1.id.clone(), bot2.id.clone()],
            },
        )
        .unwrap();

        assert_eq!(topic.bots.len(), 2);

        // Replace bots: remove bot1+bot2, add bot3
        let updated = db_update_topic_bots(&conn, &topic.id, vec![bot3.id.clone()]).unwrap();

        assert_eq!(updated.bots.len(), 1);
        assert_eq!(updated.bots[0].id, bot3.id);

        // Verify updated_at changed
        assert!(updated.updated_at >= topic.updated_at);
    }

    /// UT-TOPIC-07: Delete topic cascades to topic_bots -> no orphaned records
    #[test]
    fn test_delete_topic_cascades_to_topic_bots() {
        let conn = setup_test_db();
        let bot1 = create_test_bot(&conn, "Bot1");

        let topic = db_create_topic(
            &conn,
            CreateTopicRequest {
                title: "Deletable Topic".to_string(),
                bot_ids: vec![bot1.id.clone()],
            },
        )
        .unwrap();

        // Verify topic_bots entry exists
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM topic_bots WHERE topic_id = ?1",
                rusqlite::params![topic.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        // Delete topic
        db_delete_topic(&conn, &topic.id).unwrap();

        // Verify topic_bots entries are gone (cascaded)
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM topic_bots WHERE topic_id = ?1",
                rusqlite::params![topic.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);

        // Verify topic is gone
        let topics = db_list_topics(&conn).unwrap();
        assert!(topics.is_empty());
    }

    /// UT-TOPIC-08: Delete topic cascades to messages -> messages for that topic deleted
    #[test]
    fn test_delete_topic_cascades_to_messages() {
        let conn = setup_test_db();

        let topic = db_create_topic(
            &conn,
            CreateTopicRequest {
                title: "Topic With Messages".to_string(),
                bot_ids: vec![],
            },
        )
        .unwrap();

        // Insert messages directly via SQL (message commands aren't built yet)
        conn.execute(
            "INSERT INTO messages (id, topic_id, sender_type, content, created_at) VALUES ('m1', ?1, 'human', 'Hello!', datetime('now'))",
            rusqlite::params![topic.id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO messages (id, topic_id, sender_type, content, created_at) VALUES ('m2', ?1, 'human', 'World!', datetime('now'))",
            rusqlite::params![topic.id],
        )
        .unwrap();

        // Verify messages exist
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM messages WHERE topic_id = ?1",
                rusqlite::params![topic.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 2);

        // Delete topic
        db_delete_topic(&conn, &topic.id).unwrap();

        // Verify messages are gone (cascaded)
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM messages WHERE topic_id = ?1",
                rusqlite::params![topic.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
    }

    /// UT-TOPIC-09: Topic summary includes last_message_preview -> shows latest message content
    #[test]
    fn test_topic_summary_last_message_preview() {
        let conn = setup_test_db();

        let topic = db_create_topic(
            &conn,
            CreateTopicRequest {
                title: "Topic With Preview".to_string(),
                bot_ids: vec![],
            },
        )
        .unwrap();

        // Insert messages with different timestamps
        conn.execute(
            "INSERT INTO messages (id, topic_id, sender_type, content, created_at) VALUES ('m1', ?1, 'human', 'First message', '2024-01-01T00:00:00Z')",
            rusqlite::params![topic.id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO messages (id, topic_id, sender_type, content, created_at) VALUES ('m2', ?1, 'human', 'Latest message', '2024-06-01T00:00:00Z')",
            rusqlite::params![topic.id],
        )
        .unwrap();

        let topics = db_list_topics(&conn).unwrap();
        assert_eq!(topics.len(), 1);
        assert_eq!(
            topics[0].last_message_preview,
            Some("Latest message".to_string())
        );
    }

    /// UT-TOPIC-10: Topic summary bot_count is accurate -> reflects associated bot count
    #[test]
    fn test_topic_summary_bot_count() {
        let conn = setup_test_db();
        let bot1 = create_test_bot(&conn, "Bot1");
        let bot2 = create_test_bot(&conn, "Bot2");
        let bot3 = create_test_bot(&conn, "Bot3");

        let topic = db_create_topic(
            &conn,
            CreateTopicRequest {
                title: "Topic With 3 Bots".to_string(),
                bot_ids: vec![bot1.id.clone(), bot2.id.clone(), bot3.id.clone()],
            },
        )
        .unwrap();

        let topics = db_list_topics(&conn).unwrap();
        assert_eq!(topics.len(), 1);
        assert_eq!(topics[0].bot_count, 3);
        assert_eq!(topics[0].id, topic.id);

        // Also test a topic with no bots
        db_create_topic(
            &conn,
            CreateTopicRequest {
                title: "Empty Bot Topic".to_string(),
                bot_ids: vec![],
            },
        )
        .unwrap();

        let topics = db_list_topics(&conn).unwrap();
        // Find the empty one (most recently updated should be first)
        let empty_topic = topics.iter().find(|t| t.title == "Empty Bot Topic").unwrap();
        assert_eq!(empty_topic.bot_count, 0);
    }
}
