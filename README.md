# AI Group Chat

A desktop app where multiple AI models discuss topics together. Stop switching between Claude, Gemini, and GPT — let them debate in one place.

![License](https://img.shields.io/github/license/lionwashington/ai-group-chat)
![GitHub release](https://img.shields.io/github/v/release/lionwashington/ai-group-chat?include_prereleases)

## Features

- **Group chat with multiple AI bots** — add Claude, Gemini, GPT, or any compatible model to the same conversation
- **Sequential streaming** — bots reply one by one so each sees prior responses, streaming in real time
- **@mention for targeted replies** — `@Claude what do you think about Gemini's point?`
- **File & image upload** — attach images for vision-capable models; files are injected as text context
- **URL content fetching** — paste a URL in your message and all bots get the page content as context
- **OpenAI-compatible API** — works with any provider that speaks the OpenAI chat completions protocol
- **Topic import/export** — share conversations as `.aigc.json` files
- **Local-first** — SQLite storage, no account or cloud dependency

## How It Works

Create a **Topic** (conversation thread), add bots, and start chatting. Each human message triggers sequential API calls to each bot. Bots receive the full conversation history, so they naturally respond to each other — agreeing, disagreeing, or building on prior points.

```
┌──────────────────────────────────────────────────────────────┐
│  AI Group Chat                                               │
├───────────────┬──────────────────────────────────────────────┤
│               │                                              │
│  Topics       │  React vs Vue              [Edit Bots] [Export] │
│               │  ──────────────────────────────────────────  │
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
│  [Import]     │  │ @Claude what do you think?  [Send]     │  │
│               │  └────────────────────────────────────────┘  │
└───────────────┴──────────────────────────────────────────────┘
```

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain)
- [Node.js](https://nodejs.org/) 18+

### Install & Run

```bash
git clone https://github.com/lionwashington/ai-group-chat.git
cd ai-group-chat
npm install
npm run tauri dev
```

The app opens automatically. On first launch it creates a local SQLite database with 4 default bots.

## Add a Bot

Open **Manage Bots** from the sidebar and click **New Bot**:

| Field | Description |
|-------|-------------|
| Name | Display name shown in chat (e.g. `Claude`) |
| Base URL | API endpoint (see providers below) |
| API Key | Your key for that provider |
| Model | Model identifier (e.g. `claude-sonnet-4-5`, `gpt-4o`) |
| System Prompt | Optional persona or instructions |
| Supports Vision | Enable if the model can process images |

## Supported Providers

Any provider implementing the OpenAI chat completions API works out of the box — just set the `Base URL`:

| Provider | Base URL |
|----------|----------|
| OpenAI | `https://api.openai.com/v1` |
| OpenRouter | `https://openrouter.ai/api/v1` |
| Deepseek | `https://api.deepseek.com/v1` |
| Kimi (Moonshot) | `https://api.moonshot.cn/v1` |
| MiniMax | `https://api.minimax.chat/v1` |
| Ollama (local) | `http://localhost:11434/v1` |
| CLIProxyAPI | `http://localhost:8080/v1` |

## Tech Stack

| Layer | Technology |
|-------|-----------|
| App framework | [Tauri v2](https://v2.tauri.app/) |
| Backend | Rust (reqwest, rusqlite, tokio) |
| Frontend | React 19 + TypeScript + Vite 7 |
| UI | [shadcn/ui](https://ui.shadcn.com/) + Tailwind CSS v4 |
| State | [Zustand](https://zustand.docs.pmnd.rs/) |
| Database | SQLite (bundled, WAL mode) |

## Development

### Run tests

```bash
# Rust tests (79 tests)
cd src-tauri && cargo test

# Frontend tests (56 tests)
npm test

# E2E tests (requires built app)
npm run test:e2e
```

### Project structure

```
ai-group-chat/
├── src-tauri/              # Rust backend
│   └── src/
│       ├── commands/       # Tauri IPC commands
│       │   ├── bot.rs      # Bot CRUD
│       │   ├── topic.rs    # Topic CRUD + rename
│       │   ├── message.rs  # Message storage
│       │   ├── chat.rs     # AI orchestration + streaming
│       │   ├── attachment.rs # File/image handling
│       │   └── transfer.rs # Import/export
│       ├── db/             # SQLite schema + migrations
│       ├── ai/             # OpenAI-compatible HTTP client + SSE parser
│       └── utils/          # URL content fetcher
├── src/                    # React frontend
│   ├── components/
│   │   ├── chat/           # ChatView, MessageBubble, MessageInput
│   │   ├── sidebar/        # Topic list + context menu
│   │   ├── bot/            # Bot manager dialog
│   │   └── topic/          # Create topic dialog
│   ├── stores/appStore.ts  # Zustand state management
│   └── lib/tauri.ts        # IPC wrappers
└── docs/                   # Architecture, requirements, testing docs
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT — see [LICENSE](LICENSE) for details.
