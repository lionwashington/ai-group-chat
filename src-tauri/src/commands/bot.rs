use rusqlite::Connection;
use tauri::State;

use crate::db::DbState;
use crate::models::{Bot, CreateBotRequest, UpdateBotRequest};

// ---------------------------------------------------------------------------
// Core logic functions (testable without Tauri runtime)
// ---------------------------------------------------------------------------

pub fn db_list_bots(conn: &Connection) -> Result<Vec<Bot>, String> {
    let mut stmt = conn
        .prepare("SELECT id, name, avatar_color, base_url, api_key, model, system_prompt, supports_vision, created_at FROM bots ORDER BY created_at")
        .map_err(|e| e.to_string())?;

    let bots = stmt
        .query_map([], |row| {
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

    Ok(bots)
}

pub fn db_create_bot(conn: &Connection, req: CreateBotRequest) -> Result<Bot, String> {
    if req.name.is_empty() {
        return Err("Bot name cannot be empty".to_string());
    }
    if req.base_url.is_empty() {
        return Err("Bot base_url cannot be empty".to_string());
    }

    let id = uuid::Uuid::new_v4().to_string();
    let created_at = chrono::Utc::now().to_rfc3339();
    let avatar_color = req.avatar_color.unwrap_or_else(|| "#6366f1".to_string());
    let api_key = req.api_key.unwrap_or_default();
    let system_prompt = req.system_prompt.unwrap_or_default();
    let supports_vision = req.supports_vision.unwrap_or(false);

    conn.execute(
        "INSERT INTO bots (id, name, avatar_color, base_url, api_key, model, system_prompt, supports_vision, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            id,
            req.name,
            avatar_color,
            req.base_url,
            api_key,
            req.model,
            system_prompt,
            supports_vision as i32,
            created_at,
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(Bot {
        id,
        name: req.name,
        avatar_color,
        base_url: req.base_url,
        api_key,
        model: req.model,
        system_prompt,
        supports_vision,
        created_at,
    })
}

pub fn db_update_bot(conn: &Connection, id: &str, req: UpdateBotRequest) -> Result<Bot, String> {
    // Fetch existing bot
    let existing = conn
        .query_row(
            "SELECT id, name, avatar_color, base_url, api_key, model, system_prompt, supports_vision, created_at FROM bots WHERE id = ?1",
            rusqlite::params![id],
            |row| {
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
            },
        )
        .map_err(|e| format!("Bot not found: {}", e))?;

    // Merge with request
    let name = req.name.unwrap_or(existing.name);
    let avatar_color = req.avatar_color.unwrap_or(existing.avatar_color);
    let base_url = req.base_url.unwrap_or(existing.base_url);
    let api_key = req.api_key.unwrap_or(existing.api_key);
    let model = req.model.unwrap_or(existing.model);
    let system_prompt = req.system_prompt.unwrap_or(existing.system_prompt);
    let supports_vision = req.supports_vision.unwrap_or(existing.supports_vision);

    conn.execute(
        "UPDATE bots SET name = ?1, avatar_color = ?2, base_url = ?3, api_key = ?4, model = ?5, system_prompt = ?6, supports_vision = ?7 WHERE id = ?8",
        rusqlite::params![
            name,
            avatar_color,
            base_url,
            api_key,
            model,
            system_prompt,
            supports_vision as i32,
            id,
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(Bot {
        id: existing.id,
        name,
        avatar_color,
        base_url,
        api_key,
        model,
        system_prompt,
        supports_vision,
        created_at: existing.created_at,
    })
}

pub fn db_delete_bot(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM bots WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tauri commands (thin wrappers)
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn list_bots(db: State<DbState>) -> Result<Vec<Bot>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    db_list_bots(&conn)
}

#[tauri::command]
pub fn create_bot(db: State<DbState>, req: CreateBotRequest) -> Result<Bot, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    db_create_bot(&conn, req)
}

#[tauri::command]
pub fn update_bot(db: State<DbState>, id: String, req: UpdateBotRequest) -> Result<Bot, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    db_update_bot(&conn, &id, req)
}

#[tauri::command]
pub fn delete_bot(db: State<DbState>, id: String) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    db_delete_bot(&conn, &id)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CreateBotRequest, UpdateBotRequest};
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        crate::db::schema::run_migrations(&conn).unwrap();
        conn
    }

    fn make_full_create_request() -> CreateBotRequest {
        CreateBotRequest {
            name: "TestBot".to_string(),
            avatar_color: Some("#ff0000".to_string()),
            base_url: "https://api.openai.com/v1".to_string(),
            api_key: Some("sk-test-key".to_string()),
            model: "gpt-4".to_string(),
            system_prompt: Some("You are a helpful assistant.".to_string()),
            supports_vision: Some(true),
        }
    }

    fn make_minimal_create_request() -> CreateBotRequest {
        CreateBotRequest {
            name: "MinimalBot".to_string(),
            avatar_color: None,
            base_url: "https://api.openai.com/v1".to_string(),
            api_key: None,
            model: "gpt-3.5-turbo".to_string(),
            system_prompt: None,
            supports_vision: None,
        }
    }

    /// UT-BOT-01: Create bot with all fields -> bot saved, returned with generated ID
    #[test]
    fn test_create_bot_all_fields() {
        let conn = setup_test_db();
        let req = make_full_create_request();
        let bot = db_create_bot(&conn, req).unwrap();

        assert!(!bot.id.is_empty());
        assert_eq!(bot.name, "TestBot");
        assert_eq!(bot.avatar_color, "#ff0000");
        assert_eq!(bot.base_url, "https://api.openai.com/v1");
        assert_eq!(bot.api_key, "sk-test-key");
        assert_eq!(bot.model, "gpt-4");
        assert_eq!(bot.system_prompt, "You are a helpful assistant.");
        assert!(bot.supports_vision);
        assert!(!bot.created_at.is_empty());

        // Verify it's actually in the DB
        let bots = db_list_bots(&conn).unwrap();
        assert_eq!(bots.len(), 1);
        assert_eq!(bots[0].id, bot.id);
    }

    /// UT-BOT-02: Create bot with only required fields -> defaults applied
    #[test]
    fn test_create_bot_defaults() {
        let conn = setup_test_db();
        let req = make_minimal_create_request();
        let bot = db_create_bot(&conn, req).unwrap();

        assert!(!bot.id.is_empty());
        assert_eq!(bot.name, "MinimalBot");
        assert_eq!(bot.avatar_color, "#6366f1"); // default
        assert_eq!(bot.api_key, ""); // default
        assert_eq!(bot.system_prompt, ""); // default
        assert!(!bot.supports_vision); // default false
    }

    /// UT-BOT-03: List bots when empty -> returns empty vec
    #[test]
    fn test_list_bots_empty() {
        let conn = setup_test_db();
        let bots = db_list_bots(&conn).unwrap();
        assert!(bots.is_empty());
    }

    /// UT-BOT-04: List bots after creating 3 -> returns 3 bots, ordered by created_at
    #[test]
    fn test_list_bots_ordered() {
        let conn = setup_test_db();

        // Create 3 bots with distinct timestamps
        for i in 1..=3 {
            let req = CreateBotRequest {
                name: format!("Bot{}", i),
                avatar_color: None,
                base_url: "https://api.example.com".to_string(),
                api_key: None,
                model: "gpt-4".to_string(),
                system_prompt: None,
                supports_vision: None,
            };
            db_create_bot(&conn, req).unwrap();
        }

        let bots = db_list_bots(&conn).unwrap();
        assert_eq!(bots.len(), 3);

        // Verify ordering by created_at (ascending)
        for i in 0..bots.len() - 1 {
            assert!(bots[i].created_at <= bots[i + 1].created_at);
        }

        // Verify names
        assert_eq!(bots[0].name, "Bot1");
        assert_eq!(bots[1].name, "Bot2");
        assert_eq!(bots[2].name, "Bot3");
    }

    /// UT-BOT-05: Update bot name only -> name updated, other fields unchanged
    #[test]
    fn test_update_bot_name_only() {
        let conn = setup_test_db();
        let req = make_full_create_request();
        let bot = db_create_bot(&conn, req).unwrap();

        let update_req = UpdateBotRequest {
            name: Some("UpdatedName".to_string()),
            avatar_color: None,
            base_url: None,
            api_key: None,
            model: None,
            system_prompt: None,
            supports_vision: None,
        };

        let updated = db_update_bot(&conn, &bot.id, update_req).unwrap();
        assert_eq!(updated.name, "UpdatedName");
        // Other fields unchanged
        assert_eq!(updated.avatar_color, bot.avatar_color);
        assert_eq!(updated.base_url, bot.base_url);
        assert_eq!(updated.api_key, bot.api_key);
        assert_eq!(updated.model, bot.model);
        assert_eq!(updated.system_prompt, bot.system_prompt);
        assert_eq!(updated.supports_vision, bot.supports_vision);
        assert_eq!(updated.created_at, bot.created_at);
    }

    /// UT-BOT-06: Update bot all fields -> all fields updated
    #[test]
    fn test_update_bot_all_fields() {
        let conn = setup_test_db();
        let req = make_full_create_request();
        let bot = db_create_bot(&conn, req).unwrap();

        let update_req = UpdateBotRequest {
            name: Some("NewName".to_string()),
            avatar_color: Some("#00ff00".to_string()),
            base_url: Some("https://new-api.example.com".to_string()),
            api_key: Some("sk-new-key".to_string()),
            model: Some("gpt-4-turbo".to_string()),
            system_prompt: Some("New prompt".to_string()),
            supports_vision: Some(false),
        };

        let updated = db_update_bot(&conn, &bot.id, update_req).unwrap();
        assert_eq!(updated.name, "NewName");
        assert_eq!(updated.avatar_color, "#00ff00");
        assert_eq!(updated.base_url, "https://new-api.example.com");
        assert_eq!(updated.api_key, "sk-new-key");
        assert_eq!(updated.model, "gpt-4-turbo");
        assert_eq!(updated.system_prompt, "New prompt");
        assert!(!updated.supports_vision);
        // created_at should not change
        assert_eq!(updated.created_at, bot.created_at);
    }

    /// UT-BOT-07: Delete bot -> bot no longer in list
    #[test]
    fn test_delete_bot() {
        let conn = setup_test_db();
        let req = make_full_create_request();
        let bot = db_create_bot(&conn, req).unwrap();

        // Verify it exists
        let bots = db_list_bots(&conn).unwrap();
        assert_eq!(bots.len(), 1);

        // Delete it
        db_delete_bot(&conn, &bot.id).unwrap();

        // Verify it's gone
        let bots = db_list_bots(&conn).unwrap();
        assert!(bots.is_empty());
    }

    /// UT-BOT-08: Delete non-existent bot -> no error (idempotent)
    #[test]
    fn test_delete_nonexistent_bot() {
        let conn = setup_test_db();
        let result = db_delete_bot(&conn, "nonexistent-id");
        assert!(result.is_ok());
    }

    /// UT-BOT-09: Create bot with empty name -> should fail (validation error)
    #[test]
    fn test_create_bot_empty_name_fails() {
        let conn = setup_test_db();
        let req = CreateBotRequest {
            name: "".to_string(),
            avatar_color: None,
            base_url: "https://api.example.com".to_string(),
            api_key: None,
            model: "gpt-4".to_string(),
            system_prompt: None,
            supports_vision: None,
        };
        let result = db_create_bot(&conn, req);
        assert!(result.is_err());
    }

    /// UT-BOT-10: Create bot with empty base_url -> should fail (validation error)
    #[test]
    fn test_create_bot_empty_base_url_fails() {
        let conn = setup_test_db();
        let req = CreateBotRequest {
            name: "TestBot".to_string(),
            avatar_color: None,
            base_url: "".to_string(),
            api_key: None,
            model: "gpt-4".to_string(),
            system_prompt: None,
            supports_vision: None,
        };
        let result = db_create_bot(&conn, req);
        assert!(result.is_err());
    }
}
