use rusqlite::Connection;
use std::path::Path;
use tauri::{AppHandle, Manager, State};

use crate::db::DbState;
use crate::models::Attachment;

// ---------------------------------------------------------------------------
// Core logic functions (testable without Tauri runtime)
// ---------------------------------------------------------------------------

pub fn db_save_attachment(
    conn: &Connection,
    attachments_dir: &Path,
    message_id: &str,
    file_name: &str,
    file_data: &[u8],
    mime_type: &str,
) -> Result<Attachment, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let created_at = chrono::Utc::now().to_rfc3339();

    // Determine file_type from mime_type
    let file_type = if mime_type.starts_with("image/") {
        "image"
    } else {
        "file"
    };

    // Sanitize filename: replace non-alphanumeric chars (except . and -) with _
    let sanitized: String = file_name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '.' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();

    let stored_filename = format!("{}_{}", id, sanitized);

    // Ensure attachments directory exists
    std::fs::create_dir_all(attachments_dir).map_err(|e| e.to_string())?;

    let file_path = attachments_dir.join(&stored_filename);
    std::fs::write(&file_path, file_data).map_err(|e| e.to_string())?;

    let file_path_str = file_path.to_string_lossy().to_string();

    conn.execute(
        "INSERT INTO attachments (id, message_id, file_name, file_path, file_type, mime_type, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            id,
            message_id,
            file_name,
            file_path_str,
            file_type,
            mime_type,
            created_at,
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(Attachment {
        id,
        message_id: message_id.to_string(),
        file_name: file_name.to_string(),
        file_path: file_path_str,
        file_type: file_type.to_string(),
        mime_type: mime_type.to_string(),
        created_at,
    })
}

pub fn db_read_attachment_base64(file_path: &str) -> Result<String, String> {
    use base64::Engine;
    let bytes = std::fs::read(file_path).map_err(|e| e.to_string())?;
    Ok(base64::engine::general_purpose::STANDARD.encode(&bytes))
}

// ---------------------------------------------------------------------------
// Tauri commands (thin wrappers)
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn save_attachment(
    app: AppHandle,
    db: State<DbState>,
    message_id: String,
    file_name: String,
    file_data: Vec<u8>,
    mime_type: String,
) -> Result<Attachment, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let attachments_dir = app_data_dir.join("attachments");
    db_save_attachment(&conn, &attachments_dir, &message_id, &file_name, &file_data, &mime_type)
}

#[tauri::command]
pub fn read_attachment_base64(file_path: String) -> Result<String, String> {
    db_read_attachment_base64(&file_path)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::message::{db_list_messages, db_send_human_message};
    use crate::models::SendMessageRequest;
    use rusqlite::Connection;
    use std::path::PathBuf;

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

    fn create_test_message(conn: &Connection, topic_id: &str) -> String {
        let msg = db_send_human_message(
            conn,
            SendMessageRequest {
                topic_id: topic_id.to_string(),
                content: "Test message".to_string(),
            },
        )
        .unwrap();
        msg.id
    }

    fn temp_attachments_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("test_attachments_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// UT-ATT-01: Save image attachment -> file written, DB record with file_type="image"
    #[test]
    fn test_save_image_attachment() {
        let conn = setup_test_db();
        let topic_id = create_test_topic(&conn);
        let message_id = create_test_message(&conn, &topic_id);
        let dir = temp_attachments_dir();

        let attachment = db_save_attachment(
            &conn,
            &dir,
            &message_id,
            "photo.png",
            b"fake png data",
            "image/png",
        )
        .unwrap();

        assert!(!attachment.id.is_empty());
        assert_eq!(attachment.message_id, message_id);
        assert_eq!(attachment.file_name, "photo.png");
        assert_eq!(attachment.file_type, "image");
        assert_eq!(attachment.mime_type, "image/png");
        assert!(!attachment.created_at.is_empty());

        // Verify file was written
        assert!(Path::new(&attachment.file_path).exists());
        let contents = std::fs::read(&attachment.file_path).unwrap();
        assert_eq!(contents, b"fake png data");

        // Cleanup
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// UT-ATT-02: Save file attachment -> file written, DB record with file_type="file"
    #[test]
    fn test_save_file_attachment() {
        let conn = setup_test_db();
        let topic_id = create_test_topic(&conn);
        let message_id = create_test_message(&conn, &topic_id);
        let dir = temp_attachments_dir();

        let attachment = db_save_attachment(
            &conn,
            &dir,
            &message_id,
            "document.pdf",
            b"fake pdf data",
            "application/pdf",
        )
        .unwrap();

        assert_eq!(attachment.file_type, "file");
        assert_eq!(attachment.file_name, "document.pdf");

        // Verify file was written
        assert!(Path::new(&attachment.file_path).exists());

        // Cleanup
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// UT-ATT-03: Read attachment as base64 -> correct encoding
    #[test]
    fn test_read_attachment_base64() {
        use base64::Engine;

        let dir = temp_attachments_dir();
        let file_path = dir.join("test_file.txt");
        let data = b"Hello, World!";
        std::fs::write(&file_path, data).unwrap();

        let result = db_read_attachment_base64(file_path.to_str().unwrap()).unwrap();
        let expected = base64::engine::general_purpose::STANDARD.encode(data);
        assert_eq!(result, expected);

        // Cleanup
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// UT-ATT-04: List messages includes attachments nested
    #[test]
    fn test_list_messages_includes_attachments() {
        let conn = setup_test_db();
        let topic_id = create_test_topic(&conn);
        let message_id = create_test_message(&conn, &topic_id);
        let dir = temp_attachments_dir();

        // Save two attachments for the same message
        db_save_attachment(
            &conn,
            &dir,
            &message_id,
            "photo.png",
            b"png data",
            "image/png",
        )
        .unwrap();
        db_save_attachment(
            &conn,
            &dir,
            &message_id,
            "doc.pdf",
            b"pdf data",
            "application/pdf",
        )
        .unwrap();

        let messages = db_list_messages(&conn, &topic_id).unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].attachments.len(), 2);

        let att_names: Vec<&str> = messages[0]
            .attachments
            .iter()
            .map(|a| a.file_name.as_str())
            .collect();
        assert!(att_names.contains(&"photo.png"));
        assert!(att_names.contains(&"doc.pdf"));

        // Cleanup
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// UT-ATT-05: Delete message cascades to attachment DB records
    #[test]
    fn test_delete_message_cascades_to_attachments() {
        let conn = setup_test_db();
        let topic_id = create_test_topic(&conn);
        let message_id = create_test_message(&conn, &topic_id);
        let dir = temp_attachments_dir();

        db_save_attachment(
            &conn,
            &dir,
            &message_id,
            "photo.png",
            b"png data",
            "image/png",
        )
        .unwrap();

        // Verify attachment exists
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM attachments WHERE message_id = ?1",
                rusqlite::params![message_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        // Delete the message
        conn.execute(
            "DELETE FROM messages WHERE id = ?1",
            rusqlite::params![message_id],
        )
        .unwrap();

        // Verify attachment record is gone (cascaded)
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM attachments WHERE message_id = ?1",
                rusqlite::params![message_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);

        // Cleanup
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// UT-ATT-06: file_type determined by mime_type (image/png -> "image", text/plain -> "file")
    #[test]
    fn test_file_type_determined_by_mime_type() {
        let conn = setup_test_db();
        let topic_id = create_test_topic(&conn);
        let message_id = create_test_message(&conn, &topic_id);
        let dir = temp_attachments_dir();

        // image/png -> "image"
        let att1 = db_save_attachment(
            &conn,
            &dir,
            &message_id,
            "photo.png",
            b"data",
            "image/png",
        )
        .unwrap();
        assert_eq!(att1.file_type, "image");

        // image/jpeg -> "image"
        let att2 = db_save_attachment(
            &conn,
            &dir,
            &message_id,
            "photo.jpg",
            b"data",
            "image/jpeg",
        )
        .unwrap();
        assert_eq!(att2.file_type, "image");

        // text/plain -> "file"
        let att3 = db_save_attachment(
            &conn,
            &dir,
            &message_id,
            "notes.txt",
            b"data",
            "text/plain",
        )
        .unwrap();
        assert_eq!(att3.file_type, "file");

        // application/pdf -> "file"
        let att4 = db_save_attachment(
            &conn,
            &dir,
            &message_id,
            "doc.pdf",
            b"data",
            "application/pdf",
        )
        .unwrap();
        assert_eq!(att4.file_type, "file");

        // Cleanup
        let _ = std::fs::remove_dir_all(&dir);
    }
}
