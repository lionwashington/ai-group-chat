# AI Group Chat

A desktop app where multiple AI models discuss topics together. Stop switching between Claude, Gemini, and GPT — let them debate in one place.

## Features

- **Group chat with multiple AI bots** — add Claude, Gemini, GPT, or any compatible model to the same conversation
- **Parallel streaming responses** — all bots reply simultaneously, each streaming in real time
- **@mention for targeted replies** — `@Claude what do you think about Gemini's point?` directs a reply to a specific bot
- **File & image upload (multimodal)** — attach images for vision-capable models; files are injected as text context
- **OpenAI-compatible API** — works with any provider that speaks the OpenAI chat completions protocol
- **Local SQLite storage** — all data stays on your machine, no account or server required
- **Dark mode** — full dark theme support

## How It Works

You create a Topic (a conversation thread), add the bots you want, and start chatting. Every human message triggers parallel API calls to each bot. Bots receive the full conversation history as context, so they naturally respond to each other — agreeing, disagreeing, or building on prior points.

```
┌──────────────────────────────────────────────────────────────┐
│  AI Group Chat                                    [Bots] [Settings] │
├───────────────┬──────────────────────────────────────────────┤
│               │                                              │
│  Topics       │  React vs Vue                  [Topic Settings] │
│               │  ────────────────────────────────────────── │
│  > React vs   │  Bots: Claude | Gemini | GPT                │
│    Vue        │                                              │
│  > Arch Review│  [You] Compare the two frameworks            │
│  > Code Review│  📎 screenshot.png                           │
│               │                                              │
│               │  [Claude] Looking at the screenshot...       │
│               │  ████████░░ (streaming)                      │
│               │                                              │
│               │  [Gemini] I agree with Claude's point, but.. │
│               │  ██████████ (done)                           │
│               │                                              │
│  [+ New Topic]│  ┌────────────────────────────────────────┐  │
│               │  │ @Claude what do you think?  [Send]     │  │
│               │  └────────────────────────────────────────┘  │
└───────────────┴──────────────────────────────────────────────┘
```

## Quick Start

**Prerequisites**

- [Rust](https://rustup.rs/) (stable toolchain)
- [Node.js](https://nodejs.org/) 18 or later

**Run locally**

```bash
git clone https://github.com/your-username/ai-group-chat.git
cd ai-group-chat
npm install
npm run tauri dev
```

The app opens automatically. On first launch it creates a local SQLite database in your OS app data directory.

## Add a Bot

Open **Bot Manager** (top-right button) and click **New Bot**. Fill in:

| Field | Description |
|-------|-------------|
| Name | Display name shown in chat (e.g. `Claude`) |
| Base URL | API endpoint (see table below) |
| API Key | Your key for that provider |
| Model | Model identifier (e.g. `claude-opus-4-5`, `gpt-4o`) |
| System Prompt | Optional persona or instructions for this bot |

Enable **Supports Vision** if the model can process images.

## Supported Providers

Any provider that implements the OpenAI chat completions API works out of the box. No code changes are needed — just point `base_url` at the endpoint.

| Provider | Base URL |
|----------|----------|
| CLIProxyAPI | `http://localhost:8080/v1` |
| OpenRouter | `https://openrouter.ai/api/v1` |
| OpenAI | `https://api.openai.com/v1` |
| Kimi (Moonshot) | `https://api.moonshot.cn/v1` |
| MiniMax | `https://api.minimax.chat/v1` |
| Deepseek | `https://api.deepseek.com/v1` |
| Ollama (local) | `http://localhost:11434/v1` |

## Tech Stack

| Layer | Technology |
|-------|-----------|
| App framework | Tauri v2 |
| Backend | Rust (reqwest, rusqlite, tokio) |
| Frontend | React 19 + TypeScript + Vite |
| UI components | shadcn/ui + Tailwind CSS v4 |
| State management | Zustand |
| Database | SQLite (bundled, local) |

## Development

**Run tests**

```bash
# Rust unit tests
cd src-tauri
cargo test

# Frontend (if test suite is added)
npm test
```

**Project structure**

```
ai-group-chat/
├── src-tauri/              # Rust backend
│   └── src/
│       ├── main.rs         # Tauri entry point
│       ├── models.rs       # Shared data structures
│       ├── commands/       # Tauri IPC commands
│       │   ├── bot.rs      # Bot CRUD
│       │   ├── topic.rs    # Topic CRUD
│       │   ├── message.rs  # Messaging + streaming
│       │   └── attachment.rs
│       ├── db/             # Database layer (SQLite)
│       │   ├── schema.rs
│       │   └── migrations.rs
│       └── ai/             # AI API client
│           ├── client.rs   # OpenAI-compatible HTTP client
│           └── stream.rs   # SSE stream parser
└── src/                    # React frontend
    ├── components/
    │   ├── chat/           # ChatView, MessageBubble, MessageInput
    │   ├── sidebar/        # Topic list
    │   ├── bot/            # Bot manager dialog
    │   └── topic/          # Topic settings
    ├── stores/
    │   └── appStore.ts     # Zustand global state
    └── lib/
        └── tauri.ts        # Tauri IPC wrappers
```

## License

MIT — see [LICENSE](LICENSE) for details.
