use base64::Engine;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::{AppHandle, Manager, State};

use crate::commands::attachment::db_save_attachment;
use crate::commands::bot::db_list_bots;
use crate::commands::message::db_list_messages;
use crate::commands::topic::db_get_topic;
use crate::db::DbState;

const MAX_ATTACHMENT_SIZE: u64 = 10 * 1024 * 1024; // 10MB

// ---------------------------------------------------------------------------
// Export/Import data structures
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct TopicExport {
    pub format: String,
    pub version: u32,
    pub exported_at: String,
    pub topic: TopicMeta,
    pub bots: Vec<BotExportMeta>,
    pub messages: Vec<MessageExport>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TopicMeta {
    pub title: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BotExportMeta {
    pub name: String,
    pub avatar_color: String,
    pub model: String,
    pub system_prompt: String,
    pub supports_vision: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageExport {
    pub sender_type: String,
    pub sender_bot_name: Option<String>,
    pub content: String,
    pub created_at: String,
    pub attachments: Vec<AttachmentExport>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AttachmentExport {
    pub file_name: String,
    pub file_type: String,
    pub mime_type: String,
    pub data_base64: Option<String>,
    pub skipped: bool,
    pub skip_reason: Option<String>,
}

// ---------------------------------------------------------------------------
// Core logic functions (testable without Tauri runtime)
// ---------------------------------------------------------------------------

pub fn db_export_topic(conn: &Connection, topic_id: &str) -> Result<TopicExport, String> {
    let topic = db_get_topic(conn, topic_id)?;
    let messages = db_list_messages(conn, topic_id)?;

    let bots: Vec<BotExportMeta> = topic
        .bots
        .iter()
        .map(|b| BotExportMeta {
            name: b.name.clone(),
            avatar_color: b.avatar_color.clone(),
            model: b.model.clone(),
            system_prompt: b.system_prompt.clone(),
            supports_vision: b.supports_vision,
        })
        .collect();

    let exported_messages: Vec<MessageExport> = messages
        .iter()
        .map(|msg| {
            let sender_bot_name = msg.sender_bot_id.as_ref().and_then(|bot_id| {
                topic
                    .bots
                    .iter()
                    .find(|b| b.id == *bot_id)
                    .map(|b| b.name.clone())
            });

            let attachments: Vec<AttachmentExport> = msg
                .attachments
                .iter()
                .map(|att| {
                    let path = Path::new(&att.file_path);

                    // Check file size first
                    match std::fs::metadata(path) {
                        Ok(meta) if meta.len() > MAX_ATTACHMENT_SIZE => AttachmentExport {
                            file_name: att.file_name.clone(),
                            file_type: att.file_type.clone(),
                            mime_type: att.mime_type.clone(),
                            data_base64: None,
                            skipped: true,
                            skip_reason: Some(format!(
                                "File too large ({:.1}MB, max {}MB)",
                                meta.len() as f64 / 1_048_576.0,
                                MAX_ATTACHMENT_SIZE / 1_048_576
                            )),
                        },
                        Ok(_) => match std::fs::read(path) {
                            Ok(data) => AttachmentExport {
                                file_name: att.file_name.clone(),
                                file_type: att.file_type.clone(),
                                mime_type: att.mime_type.clone(),
                                data_base64: Some(
                                    base64::engine::general_purpose::STANDARD.encode(&data),
                                ),
                                skipped: false,
                                skip_reason: None,
                            },
                            Err(e) => AttachmentExport {
                                file_name: att.file_name.clone(),
                                file_type: att.file_type.clone(),
                                mime_type: att.mime_type.clone(),
                                data_base64: None,
                                skipped: true,
                                skip_reason: Some(format!("Failed to read file: {}", e)),
                            },
                        },
                        Err(e) => AttachmentExport {
                            file_name: att.file_name.clone(),
                            file_type: att.file_type.clone(),
                            mime_type: att.mime_type.clone(),
                            data_base64: None,
                            skipped: true,
                            skip_reason: Some(format!("File not found: {}", e)),
                        },
                    }
                })
                .collect();

            MessageExport {
                sender_type: msg.sender_type.clone(),
                sender_bot_name,
                content: msg.content.clone(),
                created_at: msg.created_at.clone(),
                attachments,
            }
        })
        .collect();

    Ok(TopicExport {
        format: "ai-group-chat-export".to_string(),
        version: 1,
        exported_at: chrono::Utc::now().to_rfc3339(),
        topic: TopicMeta {
            title: topic.title,
            created_at: topic.created_at,
        },
        bots,
        messages: exported_messages,
    })
}

pub fn db_import_topic(
    conn: &Connection,
    attachments_dir: &Path,
    data: &TopicExport,
) -> Result<String, String> {
    // Validate format
    if data.format != "ai-group-chat-export" {
        return Err(format!("Invalid format: expected 'ai-group-chat-export', got '{}'", data.format));
    }
    if data.version != 1 {
        return Err(format!("Unsupported version: {}", data.version));
    }

    // Create topic with "(imported)" suffix
    let topic_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let title = format!("{} (imported)", data.topic.title);

    conn.execute(
        "INSERT INTO topics (id, title, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![topic_id, title, data.topic.created_at, now],
    )
    .map_err(|e| e.to_string())?;

    // Match exported bots to local bots by name
    let local_bots = db_list_bots(conn)?;
    let mut matched_bot_ids: Vec<String> = Vec::new();

    // Build name -> local bot ID mapping
    let bot_name_to_id: std::collections::HashMap<&str, &str> = local_bots
        .iter()
        .map(|b| (b.name.as_str(), b.id.as_str()))
        .collect();

    for export_bot in &data.bots {
        if let Some(local_id) = bot_name_to_id.get(export_bot.name.as_str()) {
            if !matched_bot_ids.contains(&local_id.to_string()) {
                matched_bot_ids.push(local_id.to_string());
            }
        }
    }

    // Link matched bots to topic
    for bot_id in &matched_bot_ids {
        conn.execute(
            "INSERT INTO topic_bots (topic_id, bot_id) VALUES (?1, ?2)",
            rusqlite::params![topic_id, bot_id],
        )
        .map_err(|e| e.to_string())?;
    }

    // Insert messages preserving created_at order
    for msg_export in &data.messages {
        let message_id = uuid::Uuid::new_v4().to_string();

        // Resolve sender_bot_id from bot name
        let sender_bot_id = msg_export.sender_bot_name.as_ref().and_then(|name| {
            bot_name_to_id.get(name.as_str()).map(|id| id.to_string())
        });

        conn.execute(
            "INSERT INTO messages (id, topic_id, sender_type, sender_bot_id, content, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                message_id,
                topic_id,
                msg_export.sender_type,
                sender_bot_id,
                msg_export.content,
                msg_export.created_at,
            ],
        )
        .map_err(|e| e.to_string())?;

        // Import attachments
        for att_export in &msg_export.attachments {
            if att_export.skipped {
                continue;
            }
            if let Some(ref data_b64) = att_export.data_base64 {
                let file_data = base64::engine::general_purpose::STANDARD
                    .decode(data_b64)
                    .map_err(|e| format!("Failed to decode base64 attachment: {}", e))?;

                db_save_attachment(
                    conn,
                    attachments_dir,
                    &message_id,
                    &att_export.file_name,
                    &file_data,
                    &att_export.mime_type,
                )?;
            }
        }
    }

    Ok(topic_id)
}

// ---------------------------------------------------------------------------
// Tauri commands (thin wrappers)
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn export_topic(
    db: State<DbState>,
    topic_id: String,
    file_path: String,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let export_data = db_export_topic(&conn, &topic_id)?;
    let json = serde_json::to_string_pretty(&export_data)
        .map_err(|e| format!("Failed to serialize: {}", e))?;
    std::fs::write(&file_path, json).map_err(|e| format!("Failed to write file: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn import_topic(
    app: AppHandle,
    db: State<DbState>,
    file_path: String,
) -> Result<String, String> {
    let json =
        std::fs::read_to_string(&file_path).map_err(|e| format!("Failed to read file: {}", e))?;
    let data: TopicExport =
        serde_json::from_str(&json).map_err(|e| format!("Failed to parse JSON: {}", e))?;
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let attachments_dir = app_data_dir.join("attachments");
    db_import_topic(&conn, &attachments_dir, &data)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::attachment::db_save_attachment;
    use crate::commands::bot::db_create_bot;
    use crate::commands::message::{db_save_bot_message, db_send_human_message};
    use crate::commands::topic::{db_create_topic, db_get_topic};
    use crate::models::{CreateBotRequest, CreateTopicRequest, SendMessageRequest};
    use rusqlite::Connection;
    use std::path::PathBuf;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        crate::db::schema::run_migrations(&conn).unwrap();
        conn
    }

    fn create_test_bot(conn: &Connection, name: &str) -> crate::models::Bot {
        db_create_bot(
            conn,
            CreateBotRequest {
                name: name.to_string(),
                avatar_color: Some("#ff0000".to_string()),
                base_url: "http://localhost:8080/v1".to_string(),
                api_key: Some("sk-test".to_string()),
                model: "test-model".to_string(),
                system_prompt: Some("You are helpful.".to_string()),
                supports_vision: Some(true),
            },
        )
        .unwrap()
    }

    fn temp_attachments_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("test_transfer_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// UT-TRANSFER-01: Export produces valid JSON structure
    #[test]
    fn test_export_json_structure() {
        let conn = setup_test_db();
        let bot = create_test_bot(&conn, "Claude Opus 4.6");
        let topic = db_create_topic(
            &conn,
            CreateTopicRequest {
                title: "Test Topic".to_string(),
                bot_ids: vec![bot.id.clone()],
            },
        )
        .unwrap();

        db_send_human_message(
            &conn,
            SendMessageRequest {
                topic_id: topic.id.clone(),
                content: "Hello bots!".to_string(),
            },
        )
        .unwrap();

        db_save_bot_message(&conn, &topic.id, &bot.id, "Hello human!").unwrap();

        let export = db_export_topic(&conn, &topic.id).unwrap();

        assert_eq!(export.format, "ai-group-chat-export");
        assert_eq!(export.version, 1);
        assert_eq!(export.topic.title, "Test Topic");
        assert_eq!(export.bots.len(), 1);
        assert_eq!(export.bots[0].name, "Claude Opus 4.6");
        assert_eq!(export.bots[0].model, "test-model");
        assert_eq!(export.bots[0].system_prompt, "You are helpful.");
        assert!(export.bots[0].supports_vision);
        assert_eq!(export.messages.len(), 2);

        // Human message
        assert_eq!(export.messages[0].sender_type, "human");
        assert!(export.messages[0].sender_bot_name.is_none());
        assert_eq!(export.messages[0].content, "Hello bots!");

        // Bot message
        assert_eq!(export.messages[1].sender_type, "bot");
        assert_eq!(
            export.messages[1].sender_bot_name.as_deref(),
            Some("Claude Opus 4.6")
        );
        assert_eq!(export.messages[1].content, "Hello human!");

        // Verify no sensitive data
        let json = serde_json::to_string(&export).unwrap();
        assert!(!json.contains("sk-test"));
        assert!(!json.contains("localhost:8080"));
    }

    /// UT-TRANSFER-02: Export includes base64 attachments
    #[test]
    fn test_export_base64_attachments() {
        let conn = setup_test_db();
        let bot = create_test_bot(&conn, "TestBot");
        let topic = db_create_topic(
            &conn,
            CreateTopicRequest {
                title: "Attachment Test".to_string(),
                bot_ids: vec![bot.id.clone()],
            },
        )
        .unwrap();

        let msg = db_send_human_message(
            &conn,
            SendMessageRequest {
                topic_id: topic.id.clone(),
                content: "See attached".to_string(),
            },
        )
        .unwrap();

        let dir = temp_attachments_dir();
        db_save_attachment(&conn, &dir, &msg.id, "test.png", b"fake png data", "image/png")
            .unwrap();

        let export = db_export_topic(&conn, &topic.id).unwrap();
        assert_eq!(export.messages[0].attachments.len(), 1);

        let att = &export.messages[0].attachments[0];
        assert_eq!(att.file_name, "test.png");
        assert_eq!(att.file_type, "image");
        assert_eq!(att.mime_type, "image/png");
        assert!(!att.skipped);
        assert!(att.skip_reason.is_none());

        // Verify base64 decodes to original
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(att.data_base64.as_ref().unwrap())
            .unwrap();
        assert_eq!(decoded, b"fake png data");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// UT-TRANSFER-03: Export skips large files (>10MB)
    #[test]
    fn test_export_skips_large_files() {
        let conn = setup_test_db();
        let bot = create_test_bot(&conn, "TestBot");
        let topic = db_create_topic(
            &conn,
            CreateTopicRequest {
                title: "Large File Test".to_string(),
                bot_ids: vec![bot.id.clone()],
            },
        )
        .unwrap();

        let msg = db_send_human_message(
            &conn,
            SendMessageRequest {
                topic_id: topic.id.clone(),
                content: "Big file".to_string(),
            },
        )
        .unwrap();

        let dir = temp_attachments_dir();
        // Create a file >10MB
        let large_data = vec![0u8; 11 * 1024 * 1024];
        db_save_attachment(
            &conn,
            &dir,
            &msg.id,
            "huge.bin",
            &large_data,
            "application/octet-stream",
        )
        .unwrap();

        let export = db_export_topic(&conn, &topic.id).unwrap();
        let att = &export.messages[0].attachments[0];
        assert!(att.skipped);
        assert!(att.data_base64.is_none());
        assert!(att.skip_reason.as_ref().unwrap().contains("too large"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// UT-TRANSFER-04: Export handles missing file gracefully
    #[test]
    fn test_export_handles_missing_file() {
        let conn = setup_test_db();
        let bot = create_test_bot(&conn, "TestBot");
        let topic = db_create_topic(
            &conn,
            CreateTopicRequest {
                title: "Missing File Test".to_string(),
                bot_ids: vec![bot.id.clone()],
            },
        )
        .unwrap();

        let msg = db_send_human_message(
            &conn,
            SendMessageRequest {
                topic_id: topic.id.clone(),
                content: "File gone".to_string(),
            },
        )
        .unwrap();

        // Insert attachment record pointing to nonexistent file
        let att_id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO attachments (id, message_id, file_name, file_path, file_type, mime_type, created_at)
             VALUES (?1, ?2, 'ghost.txt', '/tmp/nonexistent_file_12345.txt', 'file', 'text/plain', datetime('now'))",
            rusqlite::params![att_id, msg.id],
        )
        .unwrap();

        let export = db_export_topic(&conn, &topic.id).unwrap();
        let att = &export.messages[0].attachments[0];
        assert!(att.skipped);
        assert!(att.skip_reason.as_ref().unwrap().contains("not found"));
    }

    /// UT-TRANSFER-05: Import creates topic with "(imported)" suffix
    #[test]
    fn test_import_topic_suffix() {
        let conn = setup_test_db();
        let _ = create_test_bot(&conn, "Claude Opus 4.6");
        let dir = temp_attachments_dir();

        let export_data = TopicExport {
            format: "ai-group-chat-export".to_string(),
            version: 1,
            exported_at: chrono::Utc::now().to_rfc3339(),
            topic: TopicMeta {
                title: "My Discussion".to_string(),
                created_at: "2026-03-04T10:00:00+00:00".to_string(),
            },
            bots: vec![BotExportMeta {
                name: "Claude Opus 4.6".to_string(),
                avatar_color: "#ff0000".to_string(),
                model: "test-model".to_string(),
                system_prompt: "You are helpful.".to_string(),
                supports_vision: true,
            }],
            messages: vec![MessageExport {
                sender_type: "human".to_string(),
                sender_bot_name: None,
                content: "Hello!".to_string(),
                created_at: "2026-03-04T10:00:01+00:00".to_string(),
                attachments: vec![],
            }],
        };

        let topic_id = db_import_topic(&conn, &dir, &export_data).unwrap();
        let topic = db_get_topic(&conn, &topic_id).unwrap();
        assert_eq!(topic.title, "My Discussion (imported)");
        assert_eq!(topic.bots.len(), 1);
        assert_eq!(topic.bots[0].name, "Claude Opus 4.6");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// UT-TRANSFER-06: Import matches bots by name, unmatched -> NULL sender_bot_id
    #[test]
    fn test_import_bot_matching() {
        let conn = setup_test_db();
        let _ = create_test_bot(&conn, "Claude Opus 4.6");
        let dir = temp_attachments_dir();

        let export_data = TopicExport {
            format: "ai-group-chat-export".to_string(),
            version: 1,
            exported_at: chrono::Utc::now().to_rfc3339(),
            topic: TopicMeta {
                title: "Bot Match Test".to_string(),
                created_at: "2026-03-04T10:00:00+00:00".to_string(),
            },
            bots: vec![
                BotExportMeta {
                    name: "Claude Opus 4.6".to_string(),
                    avatar_color: "#ff0000".to_string(),
                    model: "test-model".to_string(),
                    system_prompt: "".to_string(),
                    supports_vision: false,
                },
                BotExportMeta {
                    name: "NonExistentBot".to_string(),
                    avatar_color: "#00ff00".to_string(),
                    model: "unknown-model".to_string(),
                    system_prompt: "".to_string(),
                    supports_vision: false,
                },
            ],
            messages: vec![
                MessageExport {
                    sender_type: "bot".to_string(),
                    sender_bot_name: Some("Claude Opus 4.6".to_string()),
                    content: "Matched bot".to_string(),
                    created_at: "2026-03-04T10:00:01+00:00".to_string(),
                    attachments: vec![],
                },
                MessageExport {
                    sender_type: "bot".to_string(),
                    sender_bot_name: Some("NonExistentBot".to_string()),
                    content: "Unmatched bot".to_string(),
                    created_at: "2026-03-04T10:00:02+00:00".to_string(),
                    attachments: vec![],
                },
            ],
        };

        let topic_id = db_import_topic(&conn, &dir, &export_data).unwrap();

        // Verify only matched bot is linked
        let topic = db_get_topic(&conn, &topic_id).unwrap();
        assert_eq!(topic.bots.len(), 1);
        assert_eq!(topic.bots[0].name, "Claude Opus 4.6");

        // Verify messages
        let messages = crate::commands::message::db_list_messages(&conn, &topic_id).unwrap();
        assert_eq!(messages.len(), 2);
        assert!(messages[0].sender_bot_id.is_some()); // matched
        assert!(messages[1].sender_bot_id.is_none()); // unmatched

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// UT-TRANSFER-07: Import writes attachments to disk
    #[test]
    fn test_import_attachments() {
        let conn = setup_test_db();
        let dir = temp_attachments_dir();

        let file_data = b"imported file content";
        let data_b64 = base64::engine::general_purpose::STANDARD.encode(file_data);

        let export_data = TopicExport {
            format: "ai-group-chat-export".to_string(),
            version: 1,
            exported_at: chrono::Utc::now().to_rfc3339(),
            topic: TopicMeta {
                title: "Attachment Import".to_string(),
                created_at: "2026-03-04T10:00:00+00:00".to_string(),
            },
            bots: vec![],
            messages: vec![MessageExport {
                sender_type: "human".to_string(),
                sender_bot_name: None,
                content: "With attachment".to_string(),
                created_at: "2026-03-04T10:00:01+00:00".to_string(),
                attachments: vec![AttachmentExport {
                    file_name: "doc.txt".to_string(),
                    file_type: "file".to_string(),
                    mime_type: "text/plain".to_string(),
                    data_base64: Some(data_b64),
                    skipped: false,
                    skip_reason: None,
                }],
            }],
        };

        let topic_id = db_import_topic(&conn, &dir, &export_data).unwrap();
        let messages = crate::commands::message::db_list_messages(&conn, &topic_id).unwrap();
        assert_eq!(messages[0].attachments.len(), 1);

        let att = &messages[0].attachments[0];
        assert_eq!(att.file_name, "doc.txt");
        let saved_data = std::fs::read(&att.file_path).unwrap();
        assert_eq!(saved_data, file_data);

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// UT-TRANSFER-08: Import skips already-skipped attachments
    #[test]
    fn test_import_skips_skipped_attachments() {
        let conn = setup_test_db();
        let dir = temp_attachments_dir();

        let export_data = TopicExport {
            format: "ai-group-chat-export".to_string(),
            version: 1,
            exported_at: chrono::Utc::now().to_rfc3339(),
            topic: TopicMeta {
                title: "Skip Test".to_string(),
                created_at: "2026-03-04T10:00:00+00:00".to_string(),
            },
            bots: vec![],
            messages: vec![MessageExport {
                sender_type: "human".to_string(),
                sender_bot_name: None,
                content: "Skipped att".to_string(),
                created_at: "2026-03-04T10:00:01+00:00".to_string(),
                attachments: vec![AttachmentExport {
                    file_name: "huge.bin".to_string(),
                    file_type: "file".to_string(),
                    mime_type: "application/octet-stream".to_string(),
                    data_base64: None,
                    skipped: true,
                    skip_reason: Some("File too large".to_string()),
                }],
            }],
        };

        let topic_id = db_import_topic(&conn, &dir, &export_data).unwrap();
        let messages = crate::commands::message::db_list_messages(&conn, &topic_id).unwrap();
        // Skipped attachment should not be saved
        assert_eq!(messages[0].attachments.len(), 0);

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// UT-TRANSFER-09: Round-trip export -> import preserves data
    #[test]
    fn test_round_trip() {
        let conn = setup_test_db();
        let bot = create_test_bot(&conn, "RoundTripBot");
        let topic = db_create_topic(
            &conn,
            CreateTopicRequest {
                title: "Round Trip".to_string(),
                bot_ids: vec![bot.id.clone()],
            },
        )
        .unwrap();

        let msg = db_send_human_message(
            &conn,
            SendMessageRequest {
                topic_id: topic.id.clone(),
                content: "Human says hi".to_string(),
            },
        )
        .unwrap();

        let dir = temp_attachments_dir();
        db_save_attachment(
            &conn,
            &dir,
            &msg.id,
            "round.txt",
            b"round trip data",
            "text/plain",
        )
        .unwrap();

        db_save_bot_message(&conn, &topic.id, &bot.id, "Bot says hi").unwrap();

        // Export
        let export = db_export_topic(&conn, &topic.id).unwrap();
        let json = serde_json::to_string_pretty(&export).unwrap();

        // Import
        let imported: TopicExport = serde_json::from_str(&json).unwrap();
        let import_dir = temp_attachments_dir();
        let new_topic_id = db_import_topic(&conn, &import_dir, &imported).unwrap();

        // Verify
        let new_topic = db_get_topic(&conn, &new_topic_id).unwrap();
        assert_eq!(new_topic.title, "Round Trip (imported)");
        assert_eq!(new_topic.bots.len(), 1);
        assert_eq!(new_topic.bots[0].name, "RoundTripBot");

        let new_messages = crate::commands::message::db_list_messages(&conn, &new_topic_id).unwrap();
        assert_eq!(new_messages.len(), 2);
        assert_eq!(new_messages[0].content, "Human says hi");
        assert_eq!(new_messages[0].sender_type, "human");
        assert_eq!(new_messages[0].attachments.len(), 1);
        assert_eq!(new_messages[1].content, "Bot says hi");
        assert_eq!(new_messages[1].sender_type, "bot");
        assert!(new_messages[1].sender_bot_id.is_some());

        // Verify attachment content preserved
        let att = &new_messages[0].attachments[0];
        let data = std::fs::read(&att.file_path).unwrap();
        assert_eq!(data, b"round trip data");

        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::remove_dir_all(&import_dir);
    }

    /// UT-TRANSFER-10: Import rejects invalid format
    #[test]
    fn test_import_rejects_bad_format() {
        let conn = setup_test_db();
        let dir = temp_attachments_dir();

        let bad_data = TopicExport {
            format: "wrong-format".to_string(),
            version: 1,
            exported_at: chrono::Utc::now().to_rfc3339(),
            topic: TopicMeta {
                title: "Bad".to_string(),
                created_at: "2026-03-04T10:00:00+00:00".to_string(),
            },
            bots: vec![],
            messages: vec![],
        };

        let result = db_import_topic(&conn, &dir, &bad_data);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid format"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// UT-TRANSFER-11: Import rejects unsupported version
    #[test]
    fn test_import_rejects_bad_version() {
        let conn = setup_test_db();
        let dir = temp_attachments_dir();

        let bad_data = TopicExport {
            format: "ai-group-chat-export".to_string(),
            version: 99,
            exported_at: chrono::Utc::now().to_rfc3339(),
            topic: TopicMeta {
                title: "Bad".to_string(),
                created_at: "2026-03-04T10:00:00+00:00".to_string(),
            },
            bots: vec![],
            messages: vec![],
        };

        let result = db_import_topic(&conn, &dir, &bad_data);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unsupported version"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
