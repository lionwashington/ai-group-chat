mod ai;
mod commands;
mod db;
mod models;
mod utils;

use tauri::Manager;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init());

    #[cfg(debug_assertions)]
    {
        builder = builder.plugin(tauri_plugin_webdriver_automation::init());
    }

    builder
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
            commands::topic::rename_topic,
            commands::topic::delete_topic,
            commands::message::list_messages,
            commands::message::send_human_message,
            commands::message::save_bot_message,
            commands::attachment::save_attachment,
            commands::attachment::read_attachment_base64,
            commands::chat::chat_with_bots,
            commands::transfer::export_topic,
            commands::transfer::import_topic,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
