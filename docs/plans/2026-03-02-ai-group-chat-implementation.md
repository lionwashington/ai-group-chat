# AI Group Chat - Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a Tauri desktop app where multiple AI bots (via OpenAI-compatible APIs) participate in group discussions with shared context, streaming responses, and file/image support.

**Architecture:** Tauri v2 desktop app. Rust backend handles SQLite storage, HTTP requests to AI APIs, and SSE stream parsing. React + TypeScript frontend with shadcn/ui for the chat interface. All AI providers accessed through OpenAI-compatible chat completions API.

**Tech Stack:** Tauri v2, Rust (reqwest, rusqlite, serde, tokio), Vite, React 18, TypeScript, Tailwind CSS, shadcn/ui, Zustand

---

### Task 1: Project Scaffolding

**Files:**
- Create: `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/src/main.rs`
- Create: `src/main.tsx`, `src/App.tsx`
- Create: `vite.config.ts`, `tailwind.config.ts`, `tsconfig.json`

**Step 1: Create Tauri v2 project**

```bash
npm create tauri-app@latest ai-group-chat -- --template react-ts --manager npm
cd ai-group-chat
```

If running from the existing directory:
```bash
npm create tauri-app@latest . -- --template react-ts --manager npm
```

**Step 2: Install frontend dependencies**

```bash
npm install zustand react-markdown remark-gfm rehype-highlight
npm install -D @types/node
```

**Step 3: Set up shadcn/ui**

```bash
npx shadcn@latest init -d
npx shadcn@latest add button input textarea dialog dropdown-menu scroll-area separator avatar badge tooltip popover command
```

**Step 4: Add Rust dependencies**

Edit `src-tauri/Cargo.toml` to add:
```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-shell = "2"
tauri-plugin-dialog = "2"
tauri-plugin-fs = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.31", features = ["bundled"] }
reqwest = { version = "0.12", features = ["json", "stream"] }
tokio = { version = "1", features = ["full"] }
tokio-stream = "0.1"
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
base64 = "0.22"
futures = "0.3"
```

**Step 5: Verify project builds and launches**

```bash
npm run tauri dev
```

Expected: Empty Tauri window opens with Vite React default page.

**Step 6: Commit**

```bash
git add -A
git commit -m "feat: scaffold Tauri v2 project with React + TypeScript"
```

---

### Task 2: Database Schema & Migrations

**Files:**
- Create: `src-tauri/src/db/mod.rs`
- Create: `src-tauri/src/db/schema.rs`
- Modify: `src-tauri/src/main.rs`

**Step 1: Create database module**

Create `src-tauri/src/db/mod.rs`:
```rust
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
    Ok(conn)
}
```

**Step 2: Create schema with migrations**

Create `src-tauri/src/db/schema.rs`:
```rust
use rusqlite::Connection;

pub fn run_migrations(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch("
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
    ")?;
    Ok(())
}
```

**Step 3: Wire up in main.rs**

Update `src-tauri/src/main.rs`:
```rust
mod db;

use db::DbState;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let conn = db::init_db(app)?;
            app.manage(DbState(std::sync::Mutex::new(conn)));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Step 4: Verify it compiles**

```bash
npm run tauri dev
```

Expected: App launches without errors, SQLite database file created in app data directory.

**Step 5: Commit**

```bash
git add -A
git commit -m "feat: add SQLite database schema with migrations"
```

---

### Task 3: Bot CRUD (Rust Commands)

**Files:**
- Create: `src-tauri/src/commands/mod.rs`
- Create: `src-tauri/src/commands/bot.rs`
- Create: `src-tauri/src/models.rs`
- Modify: `src-tauri/src/main.rs`

**Step 1: Define data models**

Create `src-tauri/src/models.rs`:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Bot {
    pub id: String,
    pub name: String,
    pub avatar_color: String,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub system_prompt: String,
    pub supports_vision: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateBotRequest {
    pub name: String,
    pub avatar_color: Option<String>,
    pub base_url: String,
    pub api_key: Option<String>,
    pub model: String,
    pub system_prompt: Option<String>,
    pub supports_vision: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBotRequest {
    pub name: Option<String>,
    pub avatar_color: Option<String>,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub system_prompt: Option<String>,
    pub supports_vision: Option<bool>,
}
```

**Step 2: Implement bot commands**

Create `src-tauri/src/commands/mod.rs`:
```rust
pub mod bot;
```

Create `src-tauri/src/commands/bot.rs`:
```rust
use crate::db::DbState;
use crate::models::{Bot, CreateBotRequest, UpdateBotRequest};
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub fn list_bots(db: State<DbState>) -> Result<Vec<Bot>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, name, avatar_color, base_url, api_key, model, system_prompt, supports_vision, created_at FROM bots ORDER BY created_at")
        .map_err(|e| e.to_string())?;
    let bots = stmt
        .query_map([], |row| {
            Ok(Bot {
                id: row.get(0)?,
                name: row.get(1)?,
                avatar_color: row.get(2)?,
                base_url: row.get(3)?,
                api_key: row.get(4)?,
                model: row.get(5)?,
                system_prompt: row.get(6)?,
                supports_vision: row.get::<_, i32>(7)? != 0,
                created_at: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(bots)
}

#[tauri::command]
pub fn create_bot(db: State<DbState>, req: CreateBotRequest) -> Result<Bot, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let avatar_color = req.avatar_color.unwrap_or_else(|| "#6366f1".to_string());
    let api_key = req.api_key.unwrap_or_default();
    let system_prompt = req.system_prompt.unwrap_or_default();
    let supports_vision = req.supports_vision.unwrap_or(false);

    conn.execute(
        "INSERT INTO bots (id, name, avatar_color, base_url, api_key, model, system_prompt, supports_vision) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![id, req.name, avatar_color, req.base_url, api_key, req.model, system_prompt, supports_vision as i32],
    ).map_err(|e| e.to_string())?;

    let bot = Bot {
        id,
        name: req.name,
        avatar_color,
        base_url: req.base_url,
        api_key,
        model: req.model,
        system_prompt,
        supports_vision,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    Ok(bot)
}

#[tauri::command]
pub fn update_bot(db: State<DbState>, id: String, req: UpdateBotRequest) -> Result<Bot, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // Fetch existing bot
    let existing = conn.query_row(
        "SELECT id, name, avatar_color, base_url, api_key, model, system_prompt, supports_vision, created_at FROM bots WHERE id = ?1",
        [&id],
        |row| Ok(Bot {
            id: row.get(0)?,
            name: row.get(1)?,
            avatar_color: row.get(2)?,
            base_url: row.get(3)?,
            api_key: row.get(4)?,
            model: row.get(5)?,
            system_prompt: row.get(6)?,
            supports_vision: row.get::<_, i32>(7)? != 0,
            created_at: row.get(8)?,
        }),
    ).map_err(|e| e.to_string())?;

    let bot = Bot {
        id: existing.id,
        name: req.name.unwrap_or(existing.name),
        avatar_color: req.avatar_color.unwrap_or(existing.avatar_color),
        base_url: req.base_url.unwrap_or(existing.base_url),
        api_key: req.api_key.unwrap_or(existing.api_key),
        model: req.model.unwrap_or(existing.model),
        system_prompt: req.system_prompt.unwrap_or(existing.system_prompt),
        supports_vision: req.supports_vision.unwrap_or(existing.supports_vision),
        created_at: existing.created_at,
    };

    conn.execute(
        "UPDATE bots SET name=?1, avatar_color=?2, base_url=?3, api_key=?4, model=?5, system_prompt=?6, supports_vision=?7 WHERE id=?8",
        rusqlite::params![bot.name, bot.avatar_color, bot.base_url, bot.api_key, bot.model, bot.system_prompt, bot.supports_vision as i32, bot.id],
    ).map_err(|e| e.to_string())?;

    Ok(bot)
}

#[tauri::command]
pub fn delete_bot(db: State<DbState>, id: String) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM bots WHERE id = ?1", [&id])
        .map_err(|e| e.to_string())?;
    Ok(())
}
```

**Step 3: Register commands in main.rs**

```rust
mod commands;
mod db;
mod models;

use db::DbState;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let conn = db::init_db(app)?;
            app.manage(DbState(std::sync::Mutex::new(conn)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::bot::list_bots,
            commands::bot::create_bot,
            commands::bot::update_bot,
            commands::bot::delete_bot,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Step 4: Verify it compiles**

```bash
npm run tauri dev
```

Expected: Compiles without errors.

**Step 5: Commit**

```bash
git add -A
git commit -m "feat: add Bot CRUD Tauri commands with data models"
```

---

### Task 4: Topic CRUD (Rust Commands)

**Files:**
- Create: `src-tauri/src/commands/topic.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/models.rs`
- Modify: `src-tauri/src/main.rs`

**Step 1: Add Topic models**

Append to `src-tauri/src/models.rs`:
```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Topic {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub bots: Vec<Bot>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTopicRequest {
    pub title: String,
    pub bot_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TopicSummary {
    pub id: String,
    pub title: String,
    pub updated_at: String,
    pub bot_count: usize,
    pub last_message_preview: Option<String>,
}
```

**Step 2: Implement topic commands**

Create `src-tauri/src/commands/topic.rs`:
```rust
use crate::db::DbState;
use crate::models::{Bot, CreateTopicRequest, Topic, TopicSummary};
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub fn list_topics(db: State<DbState>) -> Result<Vec<TopicSummary>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT t.id, t.title, t.updated_at,
                    (SELECT COUNT(*) FROM topic_bots WHERE topic_id = t.id) as bot_count,
                    (SELECT content FROM messages WHERE topic_id = t.id ORDER BY created_at DESC LIMIT 1) as last_msg
             FROM topics t ORDER BY t.updated_at DESC"
        )
        .map_err(|e| e.to_string())?;
    let topics = stmt
        .query_map([], |row| {
            Ok(TopicSummary {
                id: row.get(0)?,
                title: row.get(1)?,
                updated_at: row.get(2)?,
                bot_count: row.get::<_, i32>(3)? as usize,
                last_message_preview: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(topics)
}

#[tauri::command]
pub fn get_topic(db: State<DbState>, id: String) -> Result<Topic, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let (tid, title, created_at, updated_at) = conn.query_row(
        "SELECT id, title, created_at, updated_at FROM topics WHERE id = ?1",
        [&id],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?)),
    ).map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare(
        "SELECT b.id, b.name, b.avatar_color, b.base_url, b.api_key, b.model, b.system_prompt, b.supports_vision, b.created_at
         FROM bots b JOIN topic_bots tb ON b.id = tb.bot_id WHERE tb.topic_id = ?1"
    ).map_err(|e| e.to_string())?;

    let bots = stmt.query_map([&id], |row| {
        Ok(Bot {
            id: row.get(0)?,
            name: row.get(1)?,
            avatar_color: row.get(2)?,
            base_url: row.get(3)?,
            api_key: row.get(4)?,
            model: row.get(5)?,
            system_prompt: row.get(6)?,
            supports_vision: row.get::<_, i32>(7)? != 0,
            created_at: row.get(8)?,
        })
    }).map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;

    Ok(Topic { id: tid, title, created_at, updated_at, bots })
}

#[tauri::command]
pub fn create_topic(db: State<DbState>, req: CreateTopicRequest) -> Result<Topic, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO topics (id, title, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![id, req.title, now, now],
    ).map_err(|e| e.to_string())?;

    for bot_id in &req.bot_ids {
        conn.execute(
            "INSERT INTO topic_bots (topic_id, bot_id) VALUES (?1, ?2)",
            rusqlite::params![id, bot_id],
        ).map_err(|e| e.to_string())?;
    }

    // Fetch the full topic with bots
    drop(conn);
    get_topic(db, id)
}

#[tauri::command]
pub fn update_topic_bots(db: State<DbState>, topic_id: String, bot_ids: Vec<String>) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM topic_bots WHERE topic_id = ?1", [&topic_id])
        .map_err(|e| e.to_string())?;
    for bot_id in &bot_ids {
        conn.execute(
            "INSERT INTO topic_bots (topic_id, bot_id) VALUES (?1, ?2)",
            rusqlite::params![topic_id, bot_id],
        ).map_err(|e| e.to_string())?;
    }
    conn.execute(
        "UPDATE topics SET updated_at = datetime('now') WHERE id = ?1",
        [&topic_id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_topic(db: State<DbState>, id: String) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM topics WHERE id = ?1", [&id])
        .map_err(|e| e.to_string())?;
    Ok(())
}
```

**Step 3: Register in mod.rs and main.rs**

Update `src-tauri/src/commands/mod.rs`:
```rust
pub mod bot;
pub mod topic;
```

Add to main.rs invoke_handler:
```rust
commands::topic::list_topics,
commands::topic::get_topic,
commands::topic::create_topic,
commands::topic::update_topic_bots,
commands::topic::delete_topic,
```

**Step 4: Verify it compiles**

```bash
npm run tauri dev
```

**Step 5: Commit**

```bash
git add -A
git commit -m "feat: add Topic CRUD Tauri commands"
```

---

### Task 5: Message Storage & Attachment Handling (Rust Commands)

**Files:**
- Create: `src-tauri/src/commands/message.rs`
- Create: `src-tauri/src/commands/attachment.rs`
- Modify: `src-tauri/src/models.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/main.rs`

**Step 1: Add Message and Attachment models**

Append to `src-tauri/src/models.rs`:
```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Attachment {
    pub id: String,
    pub message_id: String,
    pub file_name: String,
    pub file_path: String,
    pub file_type: String, // "image" | "file"
    pub mime_type: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub id: String,
    pub topic_id: String,
    pub sender_type: String, // "human" | "bot"
    pub sender_bot_id: Option<String>,
    pub content: String,
    pub created_at: String,
    pub attachments: Vec<Attachment>,
}

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub topic_id: String,
    pub content: String,
}
```

**Step 2: Implement message commands**

Create `src-tauri/src/commands/message.rs`:
```rust
use crate::db::DbState;
use crate::models::{Attachment, Message, SendMessageRequest};
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub fn list_messages(db: State<DbState>, topic_id: String) -> Result<Vec<Message>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut msg_stmt = conn
        .prepare(
            "SELECT id, topic_id, sender_type, sender_bot_id, content, created_at
             FROM messages WHERE topic_id = ?1 ORDER BY created_at ASC"
        )
        .map_err(|e| e.to_string())?;

    let mut att_stmt = conn
        .prepare(
            "SELECT id, message_id, file_name, file_path, file_type, mime_type, created_at
             FROM attachments WHERE message_id = ?1"
        )
        .map_err(|e| e.to_string())?;

    let messages = msg_stmt
        .query_map([&topic_id], |row| {
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
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut result = Vec::new();
    for mut msg in messages {
        let attachments = att_stmt
            .query_map([&msg.id], |row| {
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
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        msg.attachments = attachments;
        result.push(msg);
    }
    Ok(result)
}

#[tauri::command]
pub fn send_human_message(db: State<DbState>, req: SendMessageRequest) -> Result<Message, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO messages (id, topic_id, sender_type, content, created_at) VALUES (?1, ?2, 'human', ?3, ?4)",
        rusqlite::params![id, req.topic_id, req.content, now],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE topics SET updated_at = ?1 WHERE id = ?2",
        rusqlite::params![now, req.topic_id],
    ).map_err(|e| e.to_string())?;

    Ok(Message {
        id,
        topic_id: req.topic_id,
        sender_type: "human".to_string(),
        sender_bot_id: None,
        content: req.content,
        created_at: now,
        attachments: Vec::new(),
    })
}

#[tauri::command]
pub fn save_bot_message(db: State<DbState>, topic_id: String, bot_id: String, content: String) -> Result<Message, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO messages (id, topic_id, sender_type, sender_bot_id, content, created_at) VALUES (?1, ?2, 'bot', ?3, ?4, ?5)",
        rusqlite::params![id, topic_id, bot_id, content, now],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE topics SET updated_at = ?1 WHERE id = ?2",
        rusqlite::params![now, topic_id],
    ).map_err(|e| e.to_string())?;

    Ok(Message {
        id,
        topic_id,
        sender_type: "bot".to_string(),
        sender_bot_id: Some(bot_id),
        content,
        created_at: now,
        attachments: Vec::new(),
    })
}
```

**Step 3: Implement attachment commands**

Create `src-tauri/src/commands/attachment.rs`:
```rust
use crate::db::DbState;
use crate::models::Attachment;
use tauri::{Manager, State};
use uuid::Uuid;

#[tauri::command]
pub fn save_attachment(
    app: tauri::AppHandle,
    db: State<DbState>,
    message_id: String,
    file_name: String,
    file_data: Vec<u8>,
    mime_type: String,
) -> Result<Attachment, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();

    let file_type = if mime_type.starts_with("image/") { "image" } else { "file" };

    // Save file to app data directory
    let attachments_dir = app.path().app_data_dir().map_err(|e| e.to_string())?.join("attachments");
    std::fs::create_dir_all(&attachments_dir).map_err(|e| e.to_string())?;

    let ext = file_name.rsplit('.').next().unwrap_or("bin");
    let stored_name = format!("{}_{}.{}", id, file_name.replace(' ', "_"), ext);
    let file_path = attachments_dir.join(&stored_name);
    std::fs::write(&file_path, &file_data).map_err(|e| e.to_string())?;

    let file_path_str = file_path.to_string_lossy().to_string();

    conn.execute(
        "INSERT INTO attachments (id, message_id, file_name, file_path, file_type, mime_type) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![id, message_id, file_name, file_path_str, file_type, mime_type],
    ).map_err(|e| e.to_string())?;

    Ok(Attachment {
        id,
        message_id,
        file_name,
        file_path: file_path_str,
        file_type: file_type.to_string(),
        mime_type,
        created_at: chrono::Utc::now().to_rfc3339(),
    })
}

#[tauri::command]
pub fn read_attachment_base64(file_path: String) -> Result<String, String> {
    let data = std::fs::read(&file_path).map_err(|e| e.to_string())?;
    Ok(base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &data))
}
```

**Step 4: Update mod.rs and main.rs**

Update `src-tauri/src/commands/mod.rs`:
```rust
pub mod attachment;
pub mod bot;
pub mod message;
pub mod topic;
```

Add to main.rs invoke_handler:
```rust
commands::message::list_messages,
commands::message::send_human_message,
commands::message::save_bot_message,
commands::attachment::save_attachment,
commands::attachment::read_attachment_base64,
```

**Step 5: Verify and commit**

```bash
npm run tauri dev
git add -A
git commit -m "feat: add Message and Attachment Tauri commands"
```

---

### Task 6: AI Streaming Client (Rust)

**Files:**
- Create: `src-tauri/src/ai/mod.rs`
- Create: `src-tauri/src/ai/client.rs`
- Create: `src-tauri/src/ai/stream.rs`
- Create: `src-tauri/src/commands/chat.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/main.rs`

**Step 1: Implement OpenAI-compatible client**

Create `src-tauri/src/ai/mod.rs`:
```rust
pub mod client;
pub mod stream;
```

Create `src-tauri/src/ai/client.rs`:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Clone)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub stream: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: ChatContent,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum ChatContent {
    Text(String),
    Parts(Vec<ContentPart>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum ContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrl },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageUrl {
    pub url: String, // "data:image/png;base64,..." or URL
}

pub async fn send_chat_request(
    base_url: &str,
    api_key: &str,
    request: &ChatRequest,
) -> Result<reqwest::Response, String> {
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

    let client = reqwest::Client::new();
    let mut builder = client.post(&url)
        .header("Content-Type", "application/json");

    if !api_key.is_empty() {
        builder = builder.header("Authorization", format!("Bearer {}", api_key));
    }

    builder
        .json(request)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))
}
```

Create `src-tauri/src/ai/stream.rs`:
```rust
use futures::StreamExt;
use reqwest::Response;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct StreamChunk {
    choices: Option<Vec<StreamChoice>>,
}

#[derive(Debug, Deserialize)]
struct StreamChoice {
    delta: Option<StreamDelta>,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StreamDelta {
    content: Option<String>,
}

/// Parse SSE stream from an OpenAI-compatible API response.
/// Emits text deltas via the callback.
pub async fn process_stream<F>(response: Response, mut on_delta: F) -> Result<String, String>
where
    F: FnMut(&str),
{
    let mut full_content = String::new();
    let mut stream = response.bytes_stream();
    let mut buffer = String::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Stream read error: {}", e))?;
        let text = String::from_utf8_lossy(&chunk);
        buffer.push_str(&text);

        // Process complete SSE lines
        while let Some(pos) = buffer.find('\n') {
            let line = buffer[..pos].trim().to_string();
            buffer = buffer[pos + 1..].to_string();

            if line.is_empty() || line.starts_with(':') {
                continue;
            }

            if let Some(data) = line.strip_prefix("data: ") {
                if data.trim() == "[DONE]" {
                    return Ok(full_content);
                }

                if let Ok(chunk) = serde_json::from_str::<StreamChunk>(data) {
                    if let Some(choices) = chunk.choices {
                        for choice in choices {
                            if let Some(delta) = choice.delta {
                                if let Some(content) = delta.content {
                                    full_content.push_str(&content);
                                    on_delta(&content);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(full_content)
}
```

**Step 2: Implement chat command with streaming via Tauri events**

Create `src-tauri/src/commands/chat.rs`:
```rust
use crate::ai::client::{
    ChatContent, ChatMessage, ChatRequest, ContentPart, ImageUrl, send_chat_request,
};
use crate::ai::stream::process_stream;
use crate::db::DbState;
use crate::models::{Attachment, Bot, Message};
use serde::Serialize;
use tauri::{Emitter, Manager, State};
use uuid::Uuid;

#[derive(Debug, Serialize, Clone)]
struct StreamEvent {
    topic_id: String,
    bot_id: String,
    bot_name: String,
    delta: String,
    done: bool,
    error: Option<String>,
    message_id: Option<String>,
}

/// Build the messages array for the OpenAI-compatible API from chat history.
fn build_chat_messages(
    messages: &[Message],
    bot: &Bot,
    app: &tauri::AppHandle,
) -> Vec<ChatMessage> {
    let mut chat_messages = Vec::new();

    // Add system prompt if present
    if !bot.system_prompt.is_empty() {
        chat_messages.push(ChatMessage {
            role: "system".to_string(),
            content: ChatContent::Text(bot.system_prompt.clone()),
        });
    }

    for msg in messages {
        let role = if msg.sender_type == "human" {
            "user"
        } else {
            "assistant"
        };

        // Check for attachments with images
        if !msg.attachments.is_empty() && bot.supports_vision {
            let mut parts: Vec<ContentPart> = Vec::new();

            if !msg.content.is_empty() {
                parts.push(ContentPart::Text {
                    text: msg.content.clone(),
                });
            }

            for att in &msg.attachments {
                if att.file_type == "image" {
                    if let Ok(data) = std::fs::read(&att.file_path) {
                        let b64 = base64::Engine::encode(
                            &base64::engine::general_purpose::STANDARD,
                            &data,
                        );
                        parts.push(ContentPart::ImageUrl {
                            image_url: ImageUrl {
                                url: format!("data:{};base64,{}", att.mime_type, b64),
                            },
                        });
                    }
                } else {
                    // Read file content as text
                    if let Ok(content) = std::fs::read_to_string(&att.file_path) {
                        parts.push(ContentPart::Text {
                            text: format!(
                                "--- File: {} ---\n{}\n--- End File ---",
                                att.file_name, content
                            ),
                        });
                    }
                }
            }

            chat_messages.push(ChatMessage {
                role: role.to_string(),
                content: ChatContent::Parts(parts),
            });
        } else if !msg.attachments.is_empty() && !bot.supports_vision {
            // Non-vision bot: include file content as text, note images
            let mut text = msg.content.clone();
            for att in &msg.attachments {
                if att.file_type == "image" {
                    text.push_str(&format!("\n[Image attached: {}]", att.file_name));
                } else {
                    if let Ok(content) = std::fs::read_to_string(&att.file_path) {
                        text.push_str(&format!(
                            "\n--- File: {} ---\n{}\n--- End File ---",
                            att.file_name, content
                        ));
                    }
                }
            }
            chat_messages.push(ChatMessage {
                role: role.to_string(),
                content: ChatContent::Text(text),
            });
        } else {
            // For bot messages from OTHER bots, prefix with bot name for clarity
            let content = if msg.sender_type == "bot" && msg.sender_bot_id.as_deref() != Some(&bot.id) {
                // This is another bot's message — label it
                if let Some(ref _bot_id) = msg.sender_bot_id {
                    msg.content.clone()
                } else {
                    msg.content.clone()
                }
            } else {
                msg.content.clone()
            };

            chat_messages.push(ChatMessage {
                role: role.to_string(),
                content: ChatContent::Text(content),
            });
        }
    }

    chat_messages
}

#[tauri::command]
pub async fn chat_with_bots(
    app: tauri::AppHandle,
    db: State<'_, DbState>,
    topic_id: String,
    mentioned_bot_ids: Option<Vec<String>>,
) -> Result<(), String> {
    // 1. Get all messages for context
    let (messages, bots): (Vec<Message>, Vec<Bot>) = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;

        // Fetch messages with attachments
        let mut msg_stmt = conn.prepare(
            "SELECT id, topic_id, sender_type, sender_bot_id, content, created_at
             FROM messages WHERE topic_id = ?1 ORDER BY created_at ASC"
        ).map_err(|e| e.to_string())?;

        let mut att_stmt = conn.prepare(
            "SELECT id, message_id, file_name, file_path, file_type, mime_type, created_at
             FROM attachments WHERE message_id = ?1"
        ).map_err(|e| e.to_string())?;

        let raw_messages: Vec<Message> = msg_stmt.query_map([&topic_id], |row| {
            Ok(Message {
                id: row.get(0)?,
                topic_id: row.get(1)?,
                sender_type: row.get(2)?,
                sender_bot_id: row.get(3)?,
                content: row.get(4)?,
                created_at: row.get(5)?,
                attachments: Vec::new(),
            })
        }).map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

        let mut messages = Vec::new();
        for mut msg in raw_messages {
            let atts: Vec<Attachment> = att_stmt.query_map([&msg.id], |row| {
                Ok(Attachment {
                    id: row.get(0)?,
                    message_id: row.get(1)?,
                    file_name: row.get(2)?,
                    file_path: row.get(3)?,
                    file_type: row.get(4)?,
                    mime_type: row.get(5)?,
                    created_at: row.get(6)?,
                })
            }).map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
            msg.attachments = atts;
            messages.push(msg);
        }

        // Fetch target bots
        let bot_query = if let Some(ref ids) = mentioned_bot_ids {
            let placeholders: Vec<String> = ids.iter().enumerate().map(|(i, _)| format!("?{}", i + 2)).collect();
            format!(
                "SELECT b.id, b.name, b.avatar_color, b.base_url, b.api_key, b.model, b.system_prompt, b.supports_vision, b.created_at
                 FROM bots b WHERE b.id IN ({})", placeholders.join(",")
            )
        } else {
            "SELECT b.id, b.name, b.avatar_color, b.base_url, b.api_key, b.model, b.system_prompt, b.supports_vision, b.created_at
             FROM bots b JOIN topic_bots tb ON b.id = tb.bot_id WHERE tb.topic_id = ?1".to_string()
        };

        let mut bot_stmt = conn.prepare(&bot_query).map_err(|e| e.to_string())?;
        let bots: Vec<Bot> = if let Some(ref ids) = mentioned_bot_ids {
            let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(topic_id.clone())];
            for id in ids {
                params.push(Box::new(id.clone()));
            }
            let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
            bot_stmt.query_map(param_refs.as_slice(), |row| {
                Ok(Bot {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    avatar_color: row.get(2)?,
                    base_url: row.get(3)?,
                    api_key: row.get(4)?,
                    model: row.get(5)?,
                    system_prompt: row.get(6)?,
                    supports_vision: row.get::<_, i32>(7)? != 0,
                    created_at: row.get(8)?,
                })
            }).map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
        } else {
            bot_stmt.query_map([&topic_id], |row| {
                Ok(Bot {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    avatar_color: row.get(2)?,
                    base_url: row.get(3)?,
                    api_key: row.get(4)?,
                    model: row.get(5)?,
                    system_prompt: row.get(6)?,
                    supports_vision: row.get::<_, i32>(7)? != 0,
                    created_at: row.get(8)?,
                })
            }).map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
        };

        (messages, bots)
    };

    // 2. Spawn parallel tasks for each bot
    let mut handles = Vec::new();

    for bot in bots {
        let app_handle = app.clone();
        let messages = messages.clone();
        let topic_id = topic_id.clone();

        let handle = tokio::spawn(async move {
            let chat_messages = build_chat_messages(&messages, &bot, &app_handle);

            let request = ChatRequest {
                model: bot.model.clone(),
                messages: chat_messages,
                stream: true,
            };

            match send_chat_request(&bot.base_url, &bot.api_key, &request).await {
                Ok(response) => {
                    if !response.status().is_success() {
                        let status = response.status();
                        let body = response.text().await.unwrap_or_default();
                        let _ = app_handle.emit("chat-stream", StreamEvent {
                            topic_id: topic_id.clone(),
                            bot_id: bot.id.clone(),
                            bot_name: bot.name.clone(),
                            delta: String::new(),
                            done: true,
                            error: Some(format!("API error {}: {}", status, body)),
                            message_id: None,
                        });
                        return;
                    }

                    let app_for_stream = app_handle.clone();
                    let bot_id = bot.id.clone();
                    let bot_name = bot.name.clone();
                    let tid = topic_id.clone();

                    let result = process_stream(response, |delta| {
                        let _ = app_for_stream.emit("chat-stream", StreamEvent {
                            topic_id: tid.clone(),
                            bot_id: bot_id.clone(),
                            bot_name: bot_name.clone(),
                            delta: delta.to_string(),
                            done: false,
                            error: None,
                            message_id: None,
                        });
                    }).await;

                    match result {
                        Ok(full_content) => {
                            // Save to DB
                            let db_state = app_handle.state::<DbState>();
                            let msg_id = Uuid::new_v4().to_string();
                            let now = chrono::Utc::now().to_rfc3339();
                            {
                                let conn = db_state.0.lock().unwrap();
                                let _ = conn.execute(
                                    "INSERT INTO messages (id, topic_id, sender_type, sender_bot_id, content, created_at) VALUES (?1, ?2, 'bot', ?3, ?4, ?5)",
                                    rusqlite::params![msg_id, topic_id, bot.id, full_content, now],
                                );
                                let _ = conn.execute(
                                    "UPDATE topics SET updated_at = ?1 WHERE id = ?2",
                                    rusqlite::params![now, topic_id],
                                );
                            }

                            let _ = app_handle.emit("chat-stream", StreamEvent {
                                topic_id,
                                bot_id: bot.id,
                                bot_name: bot.name,
                                delta: String::new(),
                                done: true,
                                error: None,
                                message_id: Some(msg_id),
                            });
                        }
                        Err(e) => {
                            let _ = app_handle.emit("chat-stream", StreamEvent {
                                topic_id,
                                bot_id: bot.id,
                                bot_name: bot.name,
                                delta: String::new(),
                                done: true,
                                error: Some(e),
                                message_id: None,
                            });
                        }
                    }
                }
                Err(e) => {
                    let _ = app_handle.emit("chat-stream", StreamEvent {
                        topic_id,
                        bot_id: bot.id,
                        bot_name: bot.name,
                        delta: String::new(),
                        done: true,
                        error: Some(e),
                        message_id: None,
                    });
                }
            }
        });

        handles.push(handle);
    }

    // Wait for all bots to finish
    for handle in handles {
        let _ = handle.await;
    }

    Ok(())
}
```

**Step 3: Register in mod.rs and main.rs**

Update `src-tauri/src/commands/mod.rs`:
```rust
pub mod attachment;
pub mod bot;
pub mod chat;
pub mod message;
pub mod topic;
```

Add to main.rs:
```rust
mod ai;
```

Add to invoke_handler:
```rust
commands::chat::chat_with_bots,
```

**Step 4: Verify and commit**

```bash
npm run tauri dev
git add -A
git commit -m "feat: add AI streaming client with parallel bot responses via Tauri events"
```

---

### Task 7: Frontend - App Layout & Zustand Store

**Files:**
- Create: `src/stores/appStore.ts`
- Create: `src/lib/tauri.ts`
- Create: `src/components/sidebar/Sidebar.tsx`
- Modify: `src/App.tsx`
- Modify: `src/main.tsx`
- Modify: `src/index.css`

**Step 1: Create Tauri IPC wrappers**

Create `src/lib/tauri.ts`:
```typescript
import { invoke } from "@tauri-apps/api/core";

export interface Bot {
  id: string;
  name: string;
  avatar_color: string;
  base_url: string;
  api_key: string;
  model: string;
  system_prompt: string;
  supports_vision: boolean;
  created_at: string;
}

export interface TopicSummary {
  id: string;
  title: string;
  updated_at: string;
  bot_count: number;
  last_message_preview: string | null;
}

export interface Topic {
  id: string;
  title: string;
  created_at: string;
  updated_at: string;
  bots: Bot[];
}

export interface Attachment {
  id: string;
  message_id: string;
  file_name: string;
  file_path: string;
  file_type: "image" | "file";
  mime_type: string;
  created_at: string;
}

export interface Message {
  id: string;
  topic_id: string;
  sender_type: "human" | "bot";
  sender_bot_id: string | null;
  content: string;
  created_at: string;
  attachments: Attachment[];
}

export interface StreamEvent {
  topic_id: string;
  bot_id: string;
  bot_name: string;
  delta: string;
  done: boolean;
  error: string | null;
  message_id: string | null;
}

// Bot commands
export const listBots = () => invoke<Bot[]>("list_bots");
export const createBot = (req: { name: string; base_url: string; model: string; avatar_color?: string; api_key?: string; system_prompt?: string; supports_vision?: boolean }) =>
  invoke<Bot>("create_bot", { req });
export const updateBot = (id: string, req: Record<string, unknown>) => invoke<Bot>("update_bot", { id, req });
export const deleteBot = (id: string) => invoke<void>("delete_bot", { id });

// Topic commands
export const listTopics = () => invoke<TopicSummary[]>("list_topics");
export const getTopic = (id: string) => invoke<Topic>("get_topic", { id });
export const createTopic = (req: { title: string; bot_ids: string[] }) => invoke<Topic>("create_topic", { req });
export const updateTopicBots = (topicId: string, botIds: string[]) => invoke<void>("update_topic_bots", { topicId, botIds });
export const deleteTopic = (id: string) => invoke<void>("delete_topic", { id });

// Message commands
export const listMessages = (topicId: string) => invoke<Message[]>("list_messages", { topicId });
export const sendHumanMessage = (req: { topic_id: string; content: string }) => invoke<Message>("send_human_message", { req });
export const saveBotMessage = (topicId: string, botId: string, content: string) => invoke<Message>("save_bot_message", { topicId, botId, content });

// Attachment commands
export const saveAttachment = (messageId: string, fileName: string, fileData: number[], mimeType: string) =>
  invoke<Attachment>("save_attachment", { messageId, fileName, fileData, mimeType });

// Chat commands
export const chatWithBots = (topicId: string, mentionedBotIds?: string[]) =>
  invoke<void>("chat_with_bots", { topicId, mentionedBotIds });
```

**Step 2: Create Zustand store**

Create `src/stores/appStore.ts`:
```typescript
import { create } from "zustand";
import type { Bot, TopicSummary, Topic, Message, StreamEvent } from "../lib/tauri";

interface StreamingState {
  botId: string;
  botName: string;
  content: string;
  done: boolean;
  error: string | null;
}

interface AppState {
  // Bots
  bots: Bot[];
  setBots: (bots: Bot[]) => void;
  addBot: (bot: Bot) => void;
  removeBot: (id: string) => void;
  updateBotInStore: (bot: Bot) => void;

  // Topics
  topics: TopicSummary[];
  setTopics: (topics: TopicSummary[]) => void;
  activeTopicId: string | null;
  setActiveTopicId: (id: string | null) => void;
  activeTopic: Topic | null;
  setActiveTopic: (topic: Topic | null) => void;

  // Messages
  messages: Message[];
  setMessages: (messages: Message[]) => void;
  addMessage: (message: Message) => void;

  // Streaming
  streamingStates: Record<string, StreamingState>;
  handleStreamEvent: (event: StreamEvent) => void;
  clearStreaming: () => void;
  isAnyBotStreaming: () => boolean;
}

export const useAppStore = create<AppState>((set, get) => ({
  bots: [],
  setBots: (bots) => set({ bots }),
  addBot: (bot) => set((s) => ({ bots: [...s.bots, bot] })),
  removeBot: (id) => set((s) => ({ bots: s.bots.filter((b) => b.id !== id) })),
  updateBotInStore: (bot) =>
    set((s) => ({ bots: s.bots.map((b) => (b.id === bot.id ? bot : b)) })),

  topics: [],
  setTopics: (topics) => set({ topics }),
  activeTopicId: null,
  setActiveTopicId: (id) => set({ activeTopicId: id }),
  activeTopic: null,
  setActiveTopic: (topic) => set({ activeTopic: topic }),

  messages: [],
  setMessages: (messages) => set({ messages }),
  addMessage: (message) => set((s) => ({ messages: [...s.messages, message] })),

  streamingStates: {},
  handleStreamEvent: (event) => {
    const { activeTopicId } = get();
    if (event.topic_id !== activeTopicId) return;

    set((s) => {
      const current = s.streamingStates[event.bot_id] || {
        botId: event.bot_id,
        botName: event.bot_name,
        content: "",
        done: false,
        error: null,
      };

      return {
        streamingStates: {
          ...s.streamingStates,
          [event.bot_id]: {
            ...current,
            content: current.content + event.delta,
            done: event.done,
            error: event.error,
          },
        },
      };
    });
  },
  clearStreaming: () => set({ streamingStates: {} }),
  isAnyBotStreaming: () => {
    const states = get().streamingStates;
    return Object.values(states).some((s) => !s.done);
  },
}));
```

**Step 3: Create Sidebar component**

Create `src/components/sidebar/Sidebar.tsx`:
```tsx
import { useAppStore } from "../../stores/appStore";
import { Button } from "../ui/button";
import { ScrollArea } from "../ui/scroll-area";
import { cn } from "../../lib/utils";

export function Sidebar() {
  const { topics, activeTopicId, setActiveTopicId } = useAppStore();

  return (
    <div className="w-64 border-r flex flex-col h-full bg-muted/30">
      <div className="p-4 border-b">
        <h1 className="text-lg font-semibold">AI Group Chat</h1>
      </div>

      <ScrollArea className="flex-1">
        <div className="p-2">
          <p className="text-xs text-muted-foreground px-2 py-1">Topics</p>
          {topics.map((topic) => (
            <button
              key={topic.id}
              onClick={() => setActiveTopicId(topic.id)}
              className={cn(
                "w-full text-left px-3 py-2 rounded-md text-sm hover:bg-accent transition-colors",
                activeTopicId === topic.id && "bg-accent"
              )}
            >
              <div className="font-medium truncate">{topic.title}</div>
              {topic.last_message_preview && (
                <div className="text-xs text-muted-foreground truncate mt-0.5">
                  {topic.last_message_preview}
                </div>
              )}
            </button>
          ))}
        </div>
      </ScrollArea>

      <div className="p-3 border-t space-y-2">
        <Button variant="outline" size="sm" className="w-full" id="new-topic-btn">
          + New Topic
        </Button>
        <Button variant="ghost" size="sm" className="w-full" id="manage-bots-btn">
          Manage Bots
        </Button>
      </div>
    </div>
  );
}
```

**Step 4: Update App.tsx**

```tsx
import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { Sidebar } from "./components/sidebar/Sidebar";
import { useAppStore } from "./stores/appStore";
import { listBots, listTopics, type StreamEvent } from "./lib/tauri";

function App() {
  const { setBots, setTopics, handleStreamEvent, activeTopicId } = useAppStore();

  useEffect(() => {
    listBots().then(setBots);
    listTopics().then(setTopics);
  }, []);

  useEffect(() => {
    const unlisten = listen<StreamEvent>("chat-stream", (event) => {
      handleStreamEvent(event.payload);
    });
    return () => { unlisten.then((fn) => fn()); };
  }, []);

  return (
    <div className="flex h-screen">
      <Sidebar />
      <div className="flex-1 flex items-center justify-center text-muted-foreground">
        {activeTopicId ? (
          <p>Chat view loading...</p>
        ) : (
          <p>Select or create a topic to start chatting</p>
        )}
      </div>
    </div>
  );
}

export default App;
```

**Step 5: Update index.css for full-height layout**

Ensure `src/index.css` contains:
```css
@tailwind base;
@tailwind components;
@tailwind utilities;

html, body, #root {
  height: 100%;
  margin: 0;
  padding: 0;
}
```

**Step 6: Verify and commit**

```bash
npm run tauri dev
git add -A
git commit -m "feat: add app layout with sidebar, Zustand store, and Tauri IPC wrappers"
```

---

### Task 8: Frontend - Bot Manager Dialog

**Files:**
- Create: `src/components/bot/BotManager.tsx`
- Create: `src/components/bot/BotCard.tsx`
- Create: `src/components/bot/BotFormDialog.tsx`
- Modify: `src/App.tsx`

**Step 1: Create BotCard component**

Create `src/components/bot/BotCard.tsx`:
```tsx
import type { Bot } from "../../lib/tauri";
import { Button } from "../ui/button";
import { Badge } from "../ui/badge";

interface BotCardProps {
  bot: Bot;
  onEdit: (bot: Bot) => void;
  onDelete: (id: string) => void;
}

export function BotCard({ bot, onEdit, onDelete }: BotCardProps) {
  return (
    <div className="flex items-center justify-between p-3 border rounded-lg">
      <div className="flex items-center gap-3">
        <div
          className="w-8 h-8 rounded-full flex items-center justify-center text-white text-sm font-bold"
          style={{ backgroundColor: bot.avatar_color }}
        >
          {bot.name.charAt(0).toUpperCase()}
        </div>
        <div>
          <div className="font-medium text-sm">{bot.name}</div>
          <div className="text-xs text-muted-foreground">{bot.model}</div>
        </div>
      </div>
      <div className="flex items-center gap-2">
        {bot.supports_vision && <Badge variant="secondary">Vision</Badge>}
        <Button variant="ghost" size="sm" onClick={() => onEdit(bot)}>Edit</Button>
        <Button variant="ghost" size="sm" onClick={() => onDelete(bot.id)}>Delete</Button>
      </div>
    </div>
  );
}
```

**Step 2: Create BotFormDialog**

Create `src/components/bot/BotFormDialog.tsx`:
```tsx
import { useState, useEffect } from "react";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "../ui/dialog";
import { Button } from "../ui/button";
import { Input } from "../ui/input";
import { Textarea } from "../ui/textarea";
import type { Bot } from "../../lib/tauri";

const AVATAR_COLORS = ["#6366f1", "#ec4899", "#f59e0b", "#10b981", "#3b82f6", "#8b5cf6", "#ef4444", "#14b8a6"];

interface BotFormDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  editBot: Bot | null;
  onSubmit: (data: {
    name: string;
    base_url: string;
    api_key: string;
    model: string;
    system_prompt: string;
    avatar_color: string;
    supports_vision: boolean;
  }) => void;
}

export function BotFormDialog({ open, onOpenChange, editBot, onSubmit }: BotFormDialogProps) {
  const [name, setName] = useState("");
  const [baseUrl, setBaseUrl] = useState("");
  const [apiKey, setApiKey] = useState("");
  const [model, setModel] = useState("");
  const [systemPrompt, setSystemPrompt] = useState("");
  const [avatarColor, setAvatarColor] = useState(AVATAR_COLORS[0]);
  const [supportsVision, setSupportsVision] = useState(false);

  useEffect(() => {
    if (editBot) {
      setName(editBot.name);
      setBaseUrl(editBot.base_url);
      setApiKey(editBot.api_key);
      setModel(editBot.model);
      setSystemPrompt(editBot.system_prompt);
      setAvatarColor(editBot.avatar_color);
      setSupportsVision(editBot.supports_vision);
    } else {
      setName(""); setBaseUrl(""); setApiKey(""); setModel("");
      setSystemPrompt(""); setAvatarColor(AVATAR_COLORS[Math.floor(Math.random() * AVATAR_COLORS.length)]);
      setSupportsVision(false);
    }
  }, [editBot, open]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onSubmit({ name, base_url: baseUrl, api_key: apiKey, model, system_prompt: systemPrompt, avatar_color: avatarColor, supports_vision: supportsVision });
    onOpenChange(false);
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle>{editBot ? "Edit Bot" : "Add Bot"}</DialogTitle>
        </DialogHeader>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="text-sm font-medium">Name</label>
            <Input value={name} onChange={(e) => setName(e.target.value)} placeholder="Claude Sonnet" required />
          </div>
          <div>
            <label className="text-sm font-medium">API Base URL</label>
            <Input value={baseUrl} onChange={(e) => setBaseUrl(e.target.value)} placeholder="http://localhost:8080/v1" required />
          </div>
          <div>
            <label className="text-sm font-medium">API Key (optional)</label>
            <Input value={apiKey} onChange={(e) => setApiKey(e.target.value)} placeholder="sk-..." type="password" />
          </div>
          <div>
            <label className="text-sm font-medium">Model</label>
            <Input value={model} onChange={(e) => setModel(e.target.value)} placeholder="claude-sonnet-4-20250514" required />
          </div>
          <div>
            <label className="text-sm font-medium">System Prompt (optional)</label>
            <Textarea value={systemPrompt} onChange={(e) => setSystemPrompt(e.target.value)} placeholder="You are a helpful assistant..." rows={3} />
          </div>
          <div className="flex items-center gap-2">
            <label className="text-sm font-medium">Color</label>
            <div className="flex gap-1">
              {AVATAR_COLORS.map((color) => (
                <button
                  key={color}
                  type="button"
                  className={`w-6 h-6 rounded-full border-2 ${avatarColor === color ? "border-foreground" : "border-transparent"}`}
                  style={{ backgroundColor: color }}
                  onClick={() => setAvatarColor(color)}
                />
              ))}
            </div>
          </div>
          <div className="flex items-center gap-2">
            <input type="checkbox" id="vision" checked={supportsVision} onChange={(e) => setSupportsVision(e.target.checked)} />
            <label htmlFor="vision" className="text-sm">Supports vision (image input)</label>
          </div>
          <Button type="submit" className="w-full">
            {editBot ? "Save Changes" : "Add Bot"}
          </Button>
        </form>
      </DialogContent>
    </Dialog>
  );
}
```

**Step 3: Create BotManager**

Create `src/components/bot/BotManager.tsx`:
```tsx
import { useState } from "react";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "../ui/dialog";
import { BotCard } from "./BotCard";
import { BotFormDialog } from "./BotFormDialog";
import { Button } from "../ui/button";
import { useAppStore } from "../../stores/appStore";
import { createBot, updateBot, deleteBot, type Bot } from "../../lib/tauri";

interface BotManagerProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function BotManager({ open, onOpenChange }: BotManagerProps) {
  const { bots, addBot, removeBot, updateBotInStore } = useAppStore();
  const [formOpen, setFormOpen] = useState(false);
  const [editingBot, setEditingBot] = useState<Bot | null>(null);

  const handleCreate = async (data: Parameters<typeof createBot>[0]) => {
    const bot = await createBot(data);
    addBot(bot);
  };

  const handleEdit = (bot: Bot) => {
    setEditingBot(bot);
    setFormOpen(true);
  };

  const handleUpdate = async (data: Record<string, unknown>) => {
    if (!editingBot) return;
    const bot = await updateBot(editingBot.id, data);
    updateBotInStore(bot);
    setEditingBot(null);
  };

  const handleDelete = async (id: string) => {
    await deleteBot(id);
    removeBot(id);
  };

  return (
    <>
      <Dialog open={open} onOpenChange={onOpenChange}>
        <DialogContent className="max-w-lg">
          <DialogHeader>
            <DialogTitle>Manage Bots</DialogTitle>
          </DialogHeader>
          <div className="space-y-2 max-h-96 overflow-y-auto">
            {bots.length === 0 && (
              <p className="text-sm text-muted-foreground text-center py-4">
                No bots configured. Add one to get started.
              </p>
            )}
            {bots.map((bot) => (
              <BotCard key={bot.id} bot={bot} onEdit={handleEdit} onDelete={handleDelete} />
            ))}
          </div>
          <Button onClick={() => { setEditingBot(null); setFormOpen(true); }}>
            + Add Bot
          </Button>
        </DialogContent>
      </Dialog>

      <BotFormDialog
        open={formOpen}
        onOpenChange={setFormOpen}
        editBot={editingBot}
        onSubmit={editingBot ? handleUpdate : handleCreate}
      />
    </>
  );
}
```

**Step 4: Wire into App.tsx** — add state for BotManager dialog and connect to Sidebar buttons.

**Step 5: Verify and commit**

```bash
npm run tauri dev
git add -A
git commit -m "feat: add Bot manager UI with create/edit/delete dialogs"
```

---

### Task 9: Frontend - Topic Creation Dialog

**Files:**
- Create: `src/components/topic/CreateTopicDialog.tsx`
- Modify: `src/App.tsx`
- Modify: `src/components/sidebar/Sidebar.tsx`

**Step 1: Create CreateTopicDialog**

Create `src/components/topic/CreateTopicDialog.tsx`:
```tsx
import { useState } from "react";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "../ui/dialog";
import { Button } from "../ui/button";
import { Input } from "../ui/input";
import { useAppStore } from "../../stores/appStore";

interface CreateTopicDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onCreated: (topicId: string) => void;
}

export function CreateTopicDialog({ open, onOpenChange, onCreated }: CreateTopicDialogProps) {
  const { bots } = useAppStore();
  const [title, setTitle] = useState("");
  const [selectedBotIds, setSelectedBotIds] = useState<string[]>([]);

  const toggleBot = (id: string) => {
    setSelectedBotIds((prev) =>
      prev.includes(id) ? prev.filter((b) => b !== id) : [...prev, id]
    );
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!title.trim() || selectedBotIds.length === 0) return;

    const { createTopic } = await import("../../lib/tauri");
    const topic = await createTopic({ title: title.trim(), bot_ids: selectedBotIds });
    onCreated(topic.id);
    setTitle("");
    setSelectedBotIds([]);
    onOpenChange(false);
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle>New Topic</DialogTitle>
        </DialogHeader>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="text-sm font-medium">Topic Title</label>
            <Input value={title} onChange={(e) => setTitle(e.target.value)} placeholder="React vs Vue discussion" required />
          </div>
          <div>
            <label className="text-sm font-medium">Select Bots</label>
            {bots.length === 0 && (
              <p className="text-sm text-muted-foreground mt-1">No bots available. Add bots first.</p>
            )}
            <div className="space-y-2 mt-2">
              {bots.map((bot) => (
                <label key={bot.id} className="flex items-center gap-2 p-2 border rounded-md cursor-pointer hover:bg-accent">
                  <input
                    type="checkbox"
                    checked={selectedBotIds.includes(bot.id)}
                    onChange={() => toggleBot(bot.id)}
                  />
                  <div
                    className="w-6 h-6 rounded-full flex items-center justify-center text-white text-xs font-bold"
                    style={{ backgroundColor: bot.avatar_color }}
                  >
                    {bot.name.charAt(0).toUpperCase()}
                  </div>
                  <span className="text-sm">{bot.name}</span>
                  <span className="text-xs text-muted-foreground ml-auto">{bot.model}</span>
                </label>
              ))}
            </div>
          </div>
          <Button type="submit" className="w-full" disabled={!title.trim() || selectedBotIds.length === 0}>
            Create Topic
          </Button>
        </form>
      </DialogContent>
    </Dialog>
  );
}
```

**Step 2: Connect to Sidebar and App.tsx with state management for the dialog.**

**Step 3: Verify and commit**

```bash
npm run tauri dev
git add -A
git commit -m "feat: add Topic creation dialog with bot selection"
```

---

### Task 10: Frontend - Chat View with Streaming

**Files:**
- Create: `src/components/chat/ChatView.tsx`
- Create: `src/components/chat/MessageBubble.tsx`
- Create: `src/components/chat/StreamingMessage.tsx`
- Create: `src/components/chat/MessageInput.tsx`
- Create: `src/lib/markdown.ts`
- Modify: `src/App.tsx`

This is the most complex frontend task. Key behaviors:
- Display message history from DB
- Show multiple bot streaming responses simultaneously
- Input with @mention support and file/image upload
- Markdown rendering with syntax highlighting

**Step 1: Create markdown renderer**

Create `src/lib/markdown.ts`:
```typescript
// Markdown rendering config — used by MessageBubble and StreamingMessage
export const remarkPlugins = []; // remark-gfm loaded in component
export const rehypePlugins = []; // rehype-highlight loaded in component
```

**Step 2: Create MessageBubble**

Create `src/components/chat/MessageBubble.tsx`:
```tsx
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import rehypeHighlight from "rehype-highlight";
import type { Message, Bot } from "../../lib/tauri";

interface MessageBubbleProps {
  message: Message;
  bots: Bot[];
}

export function MessageBubble({ message, bots }: MessageBubbleProps) {
  const isHuman = message.sender_type === "human";
  const bot = bots.find((b) => b.id === message.sender_bot_id);

  return (
    <div className={`flex gap-3 ${isHuman ? "justify-end" : ""}`}>
      {!isHuman && (
        <div
          className="w-8 h-8 rounded-full flex-shrink-0 flex items-center justify-center text-white text-sm font-bold mt-1"
          style={{ backgroundColor: bot?.avatar_color || "#6b7280" }}
        >
          {bot?.name.charAt(0).toUpperCase() || "B"}
        </div>
      )}
      <div className={`max-w-[80%] ${isHuman ? "bg-primary text-primary-foreground" : "bg-muted"} rounded-lg px-4 py-2`}>
        {!isHuman && <div className="text-xs font-medium mb-1 opacity-70">{bot?.name || "Bot"}</div>}

        {/* Attachments */}
        {message.attachments.length > 0 && (
          <div className="space-y-1 mb-2">
            {message.attachments.map((att) => (
              <div key={att.id} className="text-xs bg-background/50 rounded px-2 py-1">
                {att.file_type === "image" ? "🖼 " : "📎 "}
                {att.file_name}
              </div>
            ))}
          </div>
        )}

        <div className="prose prose-sm dark:prose-invert max-w-none">
          <ReactMarkdown remarkPlugins={[remarkGfm]} rehypePlugins={[rehypeHighlight]}>
            {message.content}
          </ReactMarkdown>
        </div>
      </div>
      {isHuman && (
        <div className="w-8 h-8 rounded-full bg-foreground/10 flex-shrink-0 flex items-center justify-center text-sm font-bold mt-1">
          You
        </div>
      )}
    </div>
  );
}
```

**Step 3: Create StreamingMessage**

Create `src/components/chat/StreamingMessage.tsx`:
```tsx
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import rehypeHighlight from "rehype-highlight";

interface StreamingMessageProps {
  botName: string;
  avatarColor: string;
  content: string;
  done: boolean;
  error: string | null;
}

export function StreamingMessage({ botName, avatarColor, content, done, error }: StreamingMessageProps) {
  return (
    <div className="flex gap-3">
      <div
        className="w-8 h-8 rounded-full flex-shrink-0 flex items-center justify-center text-white text-sm font-bold mt-1"
        style={{ backgroundColor: avatarColor }}
      >
        {botName.charAt(0).toUpperCase()}
      </div>
      <div className="max-w-[80%] bg-muted rounded-lg px-4 py-2">
        <div className="text-xs font-medium mb-1 opacity-70">{botName}</div>
        {error ? (
          <div className="text-destructive text-sm">Error: {error}</div>
        ) : (
          <div className="prose prose-sm dark:prose-invert max-w-none">
            <ReactMarkdown remarkPlugins={[remarkGfm]} rehypePlugins={[rehypeHighlight]}>
              {content || " "}
            </ReactMarkdown>
            {!done && <span className="inline-block w-2 h-4 bg-foreground/50 animate-pulse ml-0.5" />}
          </div>
        )}
      </div>
    </div>
  );
}
```

**Step 4: Create MessageInput with @mention and file upload**

Create `src/components/chat/MessageInput.tsx`:
```tsx
import { useState, useRef, useCallback } from "react";
import { Button } from "../ui/button";
import { Textarea } from "../ui/textarea";
import type { Bot } from "../../lib/tauri";

interface MessageInputProps {
  bots: Bot[];
  disabled: boolean;
  onSend: (content: string, mentionedBotIds: string[], files: File[]) => void;
}

export function MessageInput({ bots, disabled, onSend }: MessageInputProps) {
  const [content, setContent] = useState("");
  const [files, setFiles] = useState<File[]>([]);
  const [showMentions, setShowMentions] = useState(false);
  const [mentionFilter, setMentionFilter] = useState("");
  const fileInputRef = useRef<HTMLInputElement>(null);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const handleChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    const value = e.target.value;
    setContent(value);

    // Check for @mention trigger
    const lastAt = value.lastIndexOf("@");
    if (lastAt >= 0 && (lastAt === 0 || value[lastAt - 1] === " ")) {
      const filter = value.slice(lastAt + 1);
      if (!filter.includes(" ")) {
        setShowMentions(true);
        setMentionFilter(filter.toLowerCase());
        return;
      }
    }
    setShowMentions(false);
  };

  const insertMention = (bot: Bot) => {
    const lastAt = content.lastIndexOf("@");
    setContent(content.slice(0, lastAt) + `@${bot.name} `);
    setShowMentions(false);
  };

  const handleSend = () => {
    if (!content.trim() && files.length === 0) return;

    // Parse @mentions from content
    const mentionedBotIds = bots
      .filter((bot) => content.includes(`@${bot.name}`))
      .map((bot) => bot.id);

    onSend(content, mentionedBotIds, files);
    setContent("");
    setFiles([]);
  };

  const handleFileSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files) {
      setFiles((prev) => [...prev, ...Array.from(e.target.files!)]);
    }
  };

  const removeFile = (index: number) => {
    setFiles((prev) => prev.filter((_, i) => i !== index));
  };

  const filteredBots = bots.filter((b) =>
    b.name.toLowerCase().includes(mentionFilter)
  );

  return (
    <div className="border-t p-4 relative">
      {/* File previews */}
      {files.length > 0 && (
        <div className="flex gap-2 mb-2 flex-wrap">
          {files.map((file, i) => (
            <div key={i} className="flex items-center gap-1 bg-muted rounded px-2 py-1 text-xs">
              {file.type.startsWith("image/") ? "🖼" : "📎"} {file.name}
              <button onClick={() => removeFile(i)} className="ml-1 text-muted-foreground hover:text-foreground">&times;</button>
            </div>
          ))}
        </div>
      )}

      {/* @mention dropdown */}
      {showMentions && filteredBots.length > 0 && (
        <div className="absolute bottom-full left-4 mb-1 bg-popover border rounded-md shadow-md p-1 z-10">
          {filteredBots.map((bot) => (
            <button
              key={bot.id}
              className="flex items-center gap-2 w-full px-3 py-1.5 text-sm hover:bg-accent rounded"
              onClick={() => insertMention(bot)}
            >
              <div
                className="w-5 h-5 rounded-full flex items-center justify-center text-white text-xs"
                style={{ backgroundColor: bot.avatar_color }}
              >
                {bot.name.charAt(0)}
              </div>
              {bot.name}
            </button>
          ))}
        </div>
      )}

      <div className="flex gap-2">
        <input
          ref={fileInputRef}
          type="file"
          multiple
          className="hidden"
          onChange={handleFileSelect}
        />
        <Button
          variant="ghost"
          size="icon"
          type="button"
          onClick={() => fileInputRef.current?.click()}
          disabled={disabled}
        >
          📎
        </Button>
        <Textarea
          value={content}
          onChange={handleChange}
          onKeyDown={handleKeyDown}
          placeholder="Type a message... (@mention a bot for targeted reply)"
          className="min-h-[44px] max-h-32 resize-none"
          rows={1}
          disabled={disabled}
        />
        <Button onClick={handleSend} disabled={disabled || (!content.trim() && files.length === 0)}>
          Send
        </Button>
      </div>
    </div>
  );
}
```

**Step 5: Create ChatView**

Create `src/components/chat/ChatView.tsx`:
```tsx
import { useEffect, useRef } from "react";
import { ScrollArea } from "../ui/scroll-area";
import { MessageBubble } from "./MessageBubble";
import { StreamingMessage } from "./StreamingMessage";
import { MessageInput } from "./MessageInput";
import { useAppStore } from "../../stores/appStore";
import {
  listMessages,
  getTopic,
  sendHumanMessage,
  saveAttachment,
  chatWithBots,
} from "../../lib/tauri";
import { Badge } from "../ui/badge";

export function ChatView() {
  const {
    activeTopicId,
    activeTopic,
    setActiveTopic,
    messages,
    setMessages,
    addMessage,
    streamingStates,
    clearStreaming,
    isAnyBotStreaming,
    bots,
  } = useAppStore();

  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!activeTopicId) return;
    clearStreaming();
    getTopic(activeTopicId).then(setActiveTopic);
    listMessages(activeTopicId).then(setMessages);
  }, [activeTopicId]);

  useEffect(() => {
    // Auto-scroll on new messages or streaming
    scrollRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, streamingStates]);

  const handleSend = async (content: string, mentionedBotIds: string[], files: File[]) => {
    if (!activeTopicId) return;

    // 1. Save human message
    const msg = await sendHumanMessage({ topic_id: activeTopicId, content });

    // 2. Save attachments
    for (const file of files) {
      const buffer = await file.arrayBuffer();
      const fileData = Array.from(new Uint8Array(buffer));
      await saveAttachment(msg.id, file.name, fileData, file.type);
    }

    // 3. Reload messages to include attachments
    const updatedMessages = await listMessages(activeTopicId);
    setMessages(updatedMessages);

    // 4. Trigger bot responses
    clearStreaming();
    const mentions = mentionedBotIds.length > 0 ? mentionedBotIds : undefined;
    chatWithBots(activeTopicId, mentions);
  };

  if (!activeTopic) {
    return (
      <div className="flex-1 flex items-center justify-center text-muted-foreground">
        Select or create a topic to start chatting
      </div>
    );
  }

  const topicBots = activeTopic.bots;
  const streaming = isAnyBotStreaming();

  return (
    <div className="flex-1 flex flex-col h-full">
      {/* Header */}
      <div className="border-b px-4 py-3 flex items-center justify-between">
        <div>
          <h2 className="font-semibold">{activeTopic.title}</h2>
          <div className="flex gap-1 mt-1">
            {topicBots.map((bot) => (
              <Badge key={bot.id} variant="secondary" style={{ borderColor: bot.avatar_color }}>
                {bot.name}
              </Badge>
            ))}
          </div>
        </div>
      </div>

      {/* Messages */}
      <ScrollArea className="flex-1 p-4">
        <div className="space-y-4">
          {messages.map((msg) => (
            <MessageBubble key={msg.id} message={msg} bots={bots} />
          ))}

          {/* Streaming bot responses */}
          {Object.values(streamingStates).map((state) => {
            const bot = bots.find((b) => b.id === state.botId);
            return (
              <StreamingMessage
                key={state.botId}
                botName={state.botName}
                avatarColor={bot?.avatar_color || "#6b7280"}
                content={state.content}
                done={state.done}
                error={state.error}
              />
            );
          })}

          <div ref={scrollRef} />
        </div>
      </ScrollArea>

      {/* Input */}
      <MessageInput
        bots={topicBots}
        disabled={streaming}
        onSend={handleSend}
      />
    </div>
  );
}
```

**Step 6: Wire ChatView into App.tsx** — replace the placeholder with `<ChatView />` when `activeTopicId` is set.

**Step 7: After all streaming finishes, reload messages from DB** — listen for all bots' `done` events, then call `listMessages` to refresh.

**Step 8: Verify and commit**

```bash
npm run tauri dev
git add -A
git commit -m "feat: add chat view with streaming messages, @mention, and file upload"
```

---

### Task 11: Integration & Polish

**Files:**
- Modify: `src/App.tsx` (wire everything together)
- Modify: `src/components/sidebar/Sidebar.tsx` (connect dialogs)
- Create: `src/components/topic/TopicSettings.tsx`
- Modify: `src/index.css` (dark mode, code highlighting)

**Step 1: Wire all dialogs and views together in App.tsx**

Ensure App.tsx manages state for:
- BotManager dialog open/close
- CreateTopicDialog open/close
- Active topic switching triggers message reload
- Stream events correctly update store and refresh on completion

**Step 2: Add TopicSettings for managing bots in a topic**

Create `src/components/topic/TopicSettings.tsx` — a dropdown/dialog to add/remove bots from the current topic.

**Step 3: Add dark mode support**

Update `src/index.css`:
```css
@media (prefers-color-scheme: dark) {
  :root {
    color-scheme: dark;
  }
}
```

And ensure shadcn/ui is configured with dark mode support in `tailwind.config.ts`.

**Step 4: Add code highlight styles**

```bash
npm install highlight.js
```

Import highlight.js theme in `src/main.tsx`:
```typescript
import "highlight.js/styles/github-dark.css";
```

**Step 5: Full end-to-end test**

1. Launch app → verify empty state
2. Add a bot (e.g., CLIProxyAPI Claude) → verify it appears in bot manager
3. Create a topic with the bot → verify it appears in sidebar
4. Send a message → verify bot streams a response
5. Add second bot, send message → verify both bots respond in parallel
6. Use @mention → verify only mentioned bot responds
7. Upload an image → verify it's sent to vision-capable bot
8. Upload a file → verify content is injected into context

**Step 6: Commit**

```bash
git add -A
git commit -m "feat: wire all components together, add dark mode and code highlighting"
```

---

### Task 12: README & Open Source Setup

**Files:**
- Create: `README.md`
- Create: `LICENSE`
- Create: `.gitignore`

**Step 1: Write README with project description, screenshots placeholder, installation instructions, usage guide, and contributing section.**

**Step 2: Add MIT LICENSE file.**

**Step 3: Ensure .gitignore covers node_modules, target/, dist/, *.db, .env files.**

**Step 4: Commit**

```bash
git add -A
git commit -m "docs: add README, LICENSE, and .gitignore for open source release"
```

---

Plan complete and saved to `docs/plans/2026-03-02-ai-group-chat-implementation.md`. Two execution options:

**1. Subagent-Driven (this session)** — I dispatch fresh subagent per task, review between tasks, fast iteration

**2. Parallel Session (separate)** — Open new session with executing-plans, batch execution with checkpoints

Which approach?
