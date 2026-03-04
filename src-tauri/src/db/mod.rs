pub mod schema;

use rusqlite::Connection;
use std::sync::Mutex;
use tauri::Manager;

pub struct DbState(pub Mutex<Connection>);

pub fn init_db(app: &tauri::App) -> Result<Connection, Box<dyn std::error::Error>> {
    let app_dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&app_dir)?;
    let db_path = app_dir.join("ai-group-chat.db");
    let conn = Connection::open(db_path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    schema::run_migrations(&conn)?;
    schema::seed_default_bots(&conn)?;
    Ok(conn)
}
