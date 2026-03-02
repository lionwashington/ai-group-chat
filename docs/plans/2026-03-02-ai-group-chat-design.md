# AI Group Chat - Design Document

## Problem Statement

When researching a topic, switching between different LLMs (Claude, Gemini, GPT) is painful:
- No shared context between models
- Manual copy-paste of conversation history
- Cannot get multiple AI perspectives on the same topic simultaneously
- Different LLMs may catch different errors, but leveraging this requires manual effort

## Solution

A desktop group chat tool where humans and AI bots coexist. The core feature is **group discussions** where multiple AI models participate simultaneously, share context, and naturally form different perspectives — including debating each other.

## Product Form

- **Tauri desktop application** (Rust backend + React frontend)
- **Target users**: Developers and power users who use multiple LLMs
- **Open-source project**
- **No server required** — all data stored locally

## Architecture

### Approach: Tauri Desktop App

Tauri app with Rust backend handling AI API calls and SQLite storage, React frontend for the chat UI. No CORS issues since Rust makes HTTP requests directly.

**Why Tauri:**
- Lightweight (~10MB vs Electron's ~100MB+)
- Rust backend handles API calls — no CORS restrictions
- SQLite built-in via Rust — zero external dependencies
- Active open-source ecosystem
- Cross-platform (macOS, Windows, Linux)

### Tech Stack

| Component | Technology |
|-----------|-----------|
| App Framework | Tauri v2 |
| Backend | Rust (reqwest for HTTP, rusqlite for DB) |
| Frontend | Vite + React + TypeScript |
| UI | shadcn/ui + Tailwind CSS |
| Database | SQLite (local file) |
| AI Protocol | OpenAI-compatible API (universal) |
| State | Zustand |

## Data Model

### Bot
- id, name, avatar_color
- base_url (e.g., "http://localhost:8080/v1", "https://openrouter.ai/api/v1")
- api_key (encrypted)
- model (e.g., "claude-sonnet-4-20250514", "gemini-2.5-pro")
- system_prompt
- supports_vision: boolean
- created_at

### Topic (Group Chat)
- id, title
- created_at, updated_at

### TopicBot (many-to-many)
- topic_id, bot_id

### Message
- id, topic_id
- sender_type: "human" | "bot"
- sender_bot_id (nullable, for bot messages)
- content (markdown)
- created_at

### Attachment
- id, message_id
- file_name, file_path (local path in app data dir)
- file_type: "image" | "file"
- mime_type
- created_at

**Key decisions:**
- **OpenAI-compatible API as universal protocol** — works with CLIProxyAPI, OpenRouter, MiniMax, Kimi, native OpenAI, and any compatible endpoint
- Same provider can have multiple Bots (e.g., Claude Opus vs Claude Sonnet with different system prompts)
- No user auth needed — single-user desktop app
- Attachments stored as local files, referenced by path

## Core Interaction Flow

### Group Discussion (Primary Feature)

```
Human types message + optional file/image attachments
    │
    ├──► Save message + attachments to SQLite
    │
    ├──► Determine target Bots:
    │    ├── @mention present → only mentioned Bot(s)
    │    └── no @mention → all Bots in Topic
    │
    ├──► Build context for each Bot:
    │    ├── Bot's system_prompt
    │    ├── Full message history of this Topic
    │    ├── Images as base64 data URLs (for vision-capable Bots)
    │    └── Files as text content injected into messages
    │
    ├──► Rust: parallel HTTP requests to each Bot's API
    │    Each request streams back via SSE/chunked response
    │
    ├──► Frontend: render multiple Bot streams simultaneously
    │    Each Bot gets its own message bubble, streaming in real-time
    │
    └──► Save all Bot responses to SQLite
```

### Interaction Modes (Hybrid)

- **Default**: All Bots in the Topic auto-reply to every human message
- **@mention**: `@Claude what do you think about Gemini's point?` → only Claude replies
- Bots see each other's replies in context, naturally forming debate

### File & Image Handling

- **Images**: Sent via OpenAI-compatible `image_url` content part (base64 encoded)
- **Files**: Content read and injected as text in the message
- **Vision fallback**: Non-vision Bots receive a text note "[Image attached: filename.png]" instead
- **Storage**: Files copied to Tauri app data directory, referenced by local path

## AI Provider Integration

All providers accessed through OpenAI-compatible chat completions API:

```
POST {base_url}/chat/completions
Authorization: Bearer {api_key}
Content-Type: application/json

{
  "model": "{model}",
  "messages": [...],
  "stream": true
}
```

### Example Bot Configurations

```
CLIProxyAPI (Claude):    base_url=http://localhost:8080/v1
CLIProxyAPI (Gemini):    base_url=http://localhost:8080/v1
OpenRouter:              base_url=https://openrouter.ai/api/v1
Kimi (Moonshot):         base_url=https://api.moonshot.cn/v1
MiniMax:                 base_url=https://api.minimax.chat/v1
Deepseek:                base_url=https://api.deepseek.com/v1
Native OpenAI:           base_url=https://api.openai.com/v1
Local Ollama:            base_url=http://localhost:11434/v1
```

Adding any new provider = just configure base_url + api_key + model. No code changes needed.

## Project Structure

```
ai-group-chat/
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── main.rs               # Tauri app entry
│   │   ├── commands/             # Tauri commands (IPC)
│   │   │   ├── mod.rs
│   │   │   ├── bot.rs            # Bot CRUD
│   │   │   ├── topic.rs          # Topic CRUD
│   │   │   ├── message.rs        # Message + streaming
│   │   │   └── attachment.rs     # File/image handling
│   │   ├── db/                   # Database layer
│   │   │   ├── mod.rs
│   │   │   ├── schema.rs         # Table definitions
│   │   │   └── migrations.rs     # Schema migrations
│   │   ├── ai/                   # AI API client
│   │   │   ├── mod.rs
│   │   │   ├── client.rs         # OpenAI-compatible HTTP client
│   │   │   └── stream.rs         # SSE stream parser
│   │   └── models.rs             # Data structures
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                          # React frontend
│   ├── App.tsx
│   ├── main.tsx
│   ├── components/
│   │   ├── chat/
│   │   │   ├── ChatView.tsx      # Main chat area
│   │   │   ├── MessageBubble.tsx # Single message display
│   │   │   ├── MessageInput.tsx  # Input with @mention + file upload
│   │   │   ├── StreamingMessage.tsx # Bot streaming response
│   │   │   └── AttachmentPreview.tsx # File/image preview
│   │   ├── sidebar/
│   │   │   ├── Sidebar.tsx       # Topic list
│   │   │   └── TopicItem.tsx     # Single topic entry
│   │   ├── bot/
│   │   │   ├── BotManager.tsx    # Bot CRUD dialog
│   │   │   └── BotCard.tsx       # Bot display card
│   │   ├── topic/
│   │   │   └── TopicSettings.tsx # Topic settings (manage bots)
│   │   └── ui/                   # shadcn/ui components
│   ├── hooks/
│   │   ├── useMessages.ts        # Message state management
│   │   └── useStreaming.ts       # Handle multiple bot streams
│   ├── stores/
│   │   └── appStore.ts           # Zustand global state
│   └── lib/
│       ├── tauri.ts              # Tauri IPC wrappers
│       └── markdown.ts           # Markdown rendering
├── package.json
└── README.md
```

## UI Layout

```
┌──────────────────────────────────────────────────────────────┐
│  AI Group Chat                                    [Bot管理] [⚙]│
├───────────────┬──────────────────────────────────────────────┤
│               │                                              │
│  Topics       │  📌 React vs Vue 讨论          [Topic设置]    │
│               │  ────────────────────────────────────────── │
│  ┌──────────┐ │  Bots: Claude | Gemini | GPT                │
│  │ React讨论 │ │                                              │
│  │ 架构评审  │ │  [You] 请比较两者优劣                         │
│  │ 代码审查  │ │  📎 screenshot.png                           │
│  └──────────┘ │                                              │
│               │  [Claude] 从这张截图来看...                    │
│               │  ████████░░ (streaming)                      │
│               │                                              │
│               │  [Gemini] 我同意 Claude 的部分观点，但...       │
│               │  ██████████ ✓                                │
│               │                                              │
│  [+ New Topic]│  ┌────────────────────────────────────────┐  │
│               │  │ @Claude 你怎么看？  [📎] [🖼] [发送]    │  │
│               │  └────────────────────────────────────────┘  │
└───────────────┴──────────────────────────────────────────────┘
```

- Left sidebar: Topic list with create button
- Right header: Active Bots in this Topic + Topic settings
- Right body: Chat messages with streaming indicators
- Input bar: @mention autocomplete + file/image upload buttons
- Each Bot message shows Bot name with colored avatar indicator

## MVP Scope (v1)

### Included:
- Bot management: create/edit/delete (base_url + api_key + model + system_prompt)
- Topic creation with Bot selection
- Group chat: all Bots reply in parallel with streaming
- @mention: target specific Bot for reply
- File upload: attach files as text context
- Image upload: send images to vision-capable models
- Markdown rendering with code highlighting
- Full conversation history as shared context
- Local SQLite storage — no server, no account

### Deferred to future versions:
- One-on-one direct chat with individual Bots
- Conversation export (markdown, JSON)
- Autonomous Bot-to-Bot debate rounds (without human trigger each time)
- Bot presets / templates (quick-add popular providers)
- Theme customization
- Keyboard shortcuts
