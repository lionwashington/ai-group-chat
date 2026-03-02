mod commands;
mod db;
mod models;

use tauri::Manager;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let conn = db::init_db(app)?;
            app.manage(db::DbState(std::sync::Mutex::new(conn)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::bot::list_bots,
            commands::bot::create_bot,
            commands::bot::update_bot,
            commands::bot::delete_bot,
            commands::topic::list_topics,
            commands::topic::get_topic,
            commands::topic::create_topic,
            commands::topic::update_topic_bots,
            commands::topic::delete_topic,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
