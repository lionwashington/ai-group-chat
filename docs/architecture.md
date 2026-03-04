# AI Group Chat - Architecture Document

> Last updated: 2026-03-04

## 1. Tech Stack

| Layer | Technology | Version |
|-------|-----------|---------|
| App Framework | Tauri | 2.x |
| Backend | Rust | stable |
| Frontend | React + TypeScript | 19.1 / 5.8 |
| Bundler | Vite | 7.x |
| CSS | Tailwind CSS | 4.x |
| UI Components | shadcn/ui + Radix UI | latest |
| State Management | Zustand | 5.x |
| Database | SQLite (rusqlite) | 0.31 |
| HTTP Client | reqwest | 0.12 |
| Async Runtime | Tokio | 1.x |
| Markdown | react-markdown + remark-gfm + rehype-highlight | latest |
| Testing (Rust) | built-in `#[test]` | - |
| Testing (Frontend) | Vitest + React Testing Library | 4.x / 16.x |
| E2E Testing | WebdriverIO | 9.x |

## 2. High-Level Architecture

```
┌─────────────────────────────────────────────────────┐
│                   Tauri Window                       │
│  ┌───────────────────────────────────────────────┐  │
│  │            React Frontend (Vite)              │  │
│  │  ┌──────────┐  ┌───────────┐  ┌───────────┐  │  │
│  │  │ Sidebar  │  │ ChatView  │  │  Dialogs   │  │  │
│  │  │          │  │           │  │ (Bot/Topic)│  │  │
│  │  └────┬─────┘  └─────┬─────┘  └─────┬─────┘  │  │
│  │       │              │               │        │  │
│  │  ┌────┴──────────────┴───────────────┴─────┐  │  │
│  │  │         Zustand Store (appStore)        │  │  │
│  │  └──────────────────┬──────────────────────┘  │  │
│  │                     │ invoke() / listen()     │  │
│  └─────────────────────┼────────────────────────┘  │
│                        │ IPC (tauri.ts wrappers)   │
├────────────────────────┼────────────────────────────┤
│  ┌─────────────────────┼────────────────────────┐  │
│  │           Rust Backend (Tauri)               │  │
│  │                     │                        │  │
│  │  ┌─────────────────────────────────────────┐ │  │
│  │  │          Commands Layer                 │ │  │
│  │  │  bot │ topic │ message │ attachment │    │ │  │
│  │  │  chat │ transfer                        │ │  │
│  │  └──────────┬─────────────────┬────────────┘ │  │
│  │             │                 │              │  │
│  │  ┌──────────┴──────┐  ┌──────┴───────────┐  │  │
│  │  │   SQLite (DB)   │  │   AI Module      │  │  │
│  │  │  schema + CRUD  │  │  client + stream  │  │  │
│  │  └─────────────────┘  └──────┬────────────┘  │  │
│  │                              │               │  │
│  └──────────────────────────────┼───────────────┘  │
│                                 │ HTTP/SSE         │
└─────────────────────────────────┼───────────────────┘
                                  │
                    ┌─────────────┴─────────────┐
                    │  OpenAI-compatible APIs    │
                    │  (Claude, Gemini, GPT...)  │
                    └───────────────────────────┘
```

## 3. Project Structure

```
ai-group-chat/
├── docs/                          # Documentation
│   ├── requirements.md
│   ├── architecture.md
│   ├── testing.md
│   └── plans/                     # Design history
├── src/                           # Frontend (React)
│   ├── main.tsx                   # Entry point
│   ├── App.tsx                    # Root component, event wiring
│   ├── vite-env.d.ts
│   ├── index.css                  # Tailwind imports
│   ├── stores/
│   │   └── appStore.ts            # Zustand global state
│   ├── lib/
│   │   ├── tauri.ts               # IPC wrappers + type definitions
│   │   └── utils.ts               # cn() utility
│   ├── components/
│   │   ├── chat/
│   │   │   ├── ChatView.tsx       # Main chat area
│   │   │   ├── MessageBubble.tsx  # Single message display
│   │   │   ├── StreamingMessage.tsx # Live streaming display
│   │   │   └── MessageInput.tsx   # Compose + @mention + upload
│   │   ├── sidebar/
│   │   │   └── Sidebar.tsx        # Topic list + actions
│   │   ├── bot/
│   │   │   ├── BotCard.tsx        # Bot display card
│   │   │   ├── BotFormDialog.tsx  # Create/edit bot form
│   │   │   └── BotManager.tsx     # Bot management dialog
│   │   ├── topic/
│   │   │   └── CreateTopicDialog.tsx # New topic form
│   │   └── ui/                    # shadcn/ui primitives (13 files)
│   └── test/
│       ├── setup.ts               # Test env (mocks, polyfills)
│       ├── fixtures.ts            # Factory functions
│       ├── smoke.test.ts
│       └── integration.test.tsx
├── src-tauri/                     # Backend (Rust)
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/
│   │   └── default.json           # Permissions: core, opener, dialog, fs
│   └── src/
│       ├── main.rs                # Binary entry
│       ├── lib.rs                 # Plugin/command registration
│       ├── models.rs              # Shared data structures
│       ├── db/
│       │   ├── mod.rs             # DB init (WAL, FK, migrations)
│       │   └── schema.rs          # CREATE TABLE + seed bots
│       ├── commands/
│       │   ├── mod.rs             # Module declarations
│       │   ├── bot.rs             # Bot CRUD
│       │   ├── topic.rs           # Topic CRUD + bot associations
│       │   ├── message.rs         # Message storage/retrieval
│       │   ├── attachment.rs      # File/image storage
│       │   ├── chat.rs            # AI orchestration + streaming
│       │   └── transfer.rs        # Topic export/import
│       ├── ai/
│       │   ├── mod.rs
│       │   ├── client.rs          # OpenAI-compatible HTTP client
│       │   └── stream.rs          # SSE parser
│       └── utils/
│           ├── mod.rs
│           └── url_fetcher.rs     # URL extraction + content fetch
├── package.json
├── tsconfig.json
├── vite.config.ts
└── vitest.config.ts
```

## 4. Database Schema

```
┌──────────────┐       ┌──────────────┐       ┌──────────────┐
│    bots      │       │  topic_bots  │       │   topics     │
├──────────────┤       ├──────────────┤       ├──────────────┤
│ id (PK)      │◄──────│ bot_id (FK)  │       │ id (PK)      │
│ name         │       │ topic_id (FK)│──────►│ title        │
│ avatar_color │       └──────────────┘       │ created_at   │
│ base_url     │              CASCADE         │ updated_at   │
│ api_key      │                              └──────┬───────┘
│ model        │                                     │
│ system_prompt│       ┌──────────────┐              │ CASCADE
│ supports_vis.│       │  messages    │              │
│ created_at   │       ├──────────────┤              │
└──────────────┘       │ id (PK)      │◄─────────────┘
       │               │ topic_id (FK)│
       │  SET NULL      │ sender_type  │  CHECK('human','bot')
       └───────────────│ sender_bot_id│
                       │ content      │
                       │ created_at   │
                       └──────┬───────┘
                              │
                              │ CASCADE
                       ┌──────┴───────┐
                       │ attachments  │
                       ├──────────────┤
                       │ id (PK)      │
                       │ message_id   │
                       │ file_name    │
                       │ file_path    │
                       │ file_type    │  CHECK('image','file')
                       │ mime_type    │
                       │ created_at   │
                       └──────────────┘
```

**Key Design Decisions:**
- UUIDs as primary keys (portable, no auto-increment conflicts)
- Foreign keys with CASCADE delete (topic deletion cleans up everything)
- `sender_bot_id` uses SET NULL on bot deletion (messages preserved, attribution lost)
- WAL mode for concurrent read access
- Timestamps as ISO 8601 strings (chrono RFC3339)

## 5. Backend Architecture

### 5.1 Two-Tier Command Pattern

Every command module follows the same pattern:

```
┌──────────────────────────────────┐
│   db_* functions (pure logic)    │  Takes &Connection, returns Result<T, String>
│   Testable without Tauri runtime │  All business logic lives here
└──────────────┬───────────────────┘
               │ called by
┌──────────────┴───────────────────┐
│  #[tauri::command] wrappers      │  Acquires Mutex lock, resolves paths,
│  Thin, no business logic         │  forwards to db_* function
└──────────────────────────────────┘
```

This pattern enables comprehensive unit testing without the Tauri runtime.

### 5.2 Command Modules

| Module | Commands | Purpose |
|--------|----------|---------|
| `bot` | `list_bots`, `create_bot`, `update_bot`, `delete_bot` | Bot CRUD |
| `topic` | `list_topics`, `get_topic`, `create_topic`, `update_topic_bots`, `rename_topic`, `delete_topic` | Topic CRUD |
| `message` | `list_messages`, `send_human_message`, `save_bot_message` | Message storage |
| `attachment` | `save_attachment`, `read_attachment_base64` | File handling |
| `chat` | `chat_with_bots` | AI orchestration |
| `transfer` | `export_topic`, `import_topic` | Import/export |

### 5.3 AI Chat Flow

```
User sends message
       │
       ▼
  chat_with_bots(topic_id, bot_ids?)
       │
       ├─ Load topic bots (filter by @mentioned bot_ids if any)
       ├─ Fetch URL content from human messages (shared cache)
       ├─ Release DB Mutex
       │
       ▼
  For each bot (SEQUENTIAL):
       │
       ├─ Acquire DB lock → load messages → release
       ├─ build_chat_messages()
       │    ├─ System prompt: "You are {name}. Other participants: ..."
       │    ├─ Map messages:
       │    │   human → role: "user"
       │    │   own bot → role: "assistant"
       │    │   other bots → role: "user" + "[BotName]: " prefix
       │    ├─ Handle attachments:
       │    │   images → base64 data URL (vision) or [Image: name] (non-vision)
       │    │   files → read content, truncate 15k chars, code block
       │    └─ Append URL content to human messages
       │
       ├─ Emit "thinking" StreamEvent
       ├─ send_chat_request() → SSE stream
       │    ├─ On 429: retry with backoff (1s, 2s, 4s), emit "retrying" status
       │    └─ On error: emit error event
       ├─ process_stream() → emit delta events
       └─ Save complete response to DB
```

### 5.4 Import/Export Flow

**Export:**
```
db_export_topic(conn, topic_id)
  → Load topic + bots (strip api_key, base_url)
  → Load messages + attachments
  → For each attachment:
      file >10MB → skip (with reason)
      file missing → skip (with reason)
      otherwise → base64 encode
  → Return TopicExport struct → serialize to JSON → write to file
```

**Import:**
```
import_topic(file_path)
  → Read + parse JSON → validate format + version
  → Create topic: "{title} (imported)"
  → Match exported bot names → local bots by name
  → Link matched bots to topic
  → Insert messages (preserving created_at)
      sender_bot_name matched → sender_bot_id set
      unmatched → sender_bot_id NULL
  → Decode + write attachments (skip already-skipped)
  → Return new topic_id
```

## 6. Frontend Architecture

### 6.1 State Management (Zustand)

```
appStore
├── Bots:      bots[], setBots, addBot, removeBot, updateBotInStore
├── Topics:    topics[], setTopics, activeTopicId, setActiveTopicId
├── Active:    activeTopic, setActiveTopic
├── Messages:  messages[], setMessages, addMessage
└── Streaming: streamingStates{}, handleStreamEvent, clearStreaming, isAnyBotStreaming
```

**Streaming State per bot:**
```typescript
{
  botId: string
  botName: string
  content: string     // accumulated deltas
  done: boolean
  error: string | null
  status: string | null    // "thinking" | "retrying" | null
  retryInfo: string | null // "Rate limited, retrying (1/3)..."
}
```

### 6.2 IPC Layer

All frontend-backend communication goes through `src/lib/tauri.ts`:

```
Frontend Component
      │ calls
      ▼
  tauri.ts wrapper  (exportTopic, importTopic, etc.)
      │ invoke()
      ▼
  Tauri IPC bridge
      │
      ▼
  Rust #[tauri::command]
```

For streaming, the backend emits events and the frontend listens:

```
Rust: app.emit("chat-stream", StreamEvent)
           │
           ▼
  App.tsx: listen<StreamEvent>("chat-stream", callback)
           │
           ▼
  appStore.handleStreamEvent(payload)
```

### 6.3 Component Tree

```
App
├── Sidebar
│   ├── Topic list (click to select, double-click to rename, hover for delete)
│   └── Actions: New Topic, Import Topic, Manage Bots
├── Resize Handle (drag 200-500px)
├── ChatView (or empty state)
│   ├── Header (title + bot badges + export button)
│   ├── Message list (auto-scroll)
│   │   ├── MessageBubble (per message, memoized)
│   │   └── StreamingMessage (per active bot, memoized)
│   └── MessageInput
│       ├── @mention autocomplete
│       └── File upload preview
├── BotManager (dialog)
│   ├── BotCard (per bot)
│   └── BotFormDialog (create/edit)
└── CreateTopicDialog
```

## 7. Data Flow Examples

### Sending a Message

```
1. User types + clicks Send
2. ChatView.handleSend()
3. → sendHumanMessage(topic_id, content)    [IPC]
4. → saveAttachment() for each file          [IPC]
5. → listMessages(topic_id)                  [IPC, reload]
6. → chatWithBots(topic_id, bot_ids?)        [IPC]
7.    Backend: sequential bot processing...
8.    → emit("chat-stream", {delta})          [Event, per token]
9.    → appStore.handleStreamEvent()
10.   → StreamingMessage re-renders
11. All bots done → listMessages() reload from DB
```

### Importing a Topic

```
1. User clicks "Import Topic" in sidebar
2. App.handleImportTopic()
3. → open() native file dialog               [Dialog plugin]
4. → importTopic(filePath)                    [IPC]
5.    Backend: validate, create topic, match bots, insert messages
6. → loadTopics()                             [IPC, refresh list]
7. → setActiveTopicId(newTopicId)             [Navigate]
```

## 8. Security Considerations

- **API keys** stored locally in SQLite, never exported in `.aigc.json`
- **Base URLs** never included in exports
- **Attachment size limit** (10MB) prevents export file bloat
- **File path sanitization** on attachment storage (alphanumeric + `.` + `-` only)
- **No CSP restrictions** currently (trusting local context)
- **All HTTP from Rust** (no browser CORS issues, no exposed credentials in frontend)

## 9. Code Statistics

| Area | Files | Lines |
|------|-------|-------|
| Rust backend | 17 | ~4,300 |
| Frontend (app code) | ~20 | ~1,860 |
| Frontend (shadcn/ui) | 13 | ~1,280 |
| Frontend (tests) | 11 | ~1,670 |
| **Total** | **~61** | **~9,100** |
