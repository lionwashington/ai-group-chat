# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.1.0] - 2026-03-04

Initial release of AI Group Chat — a desktop app for multi-AI group conversations.

### Bot Management
- Create, edit, and delete bots with name, avatar color, base URL, API key, model, system prompt, and vision support flag
- OpenAI-compatible API protocol — works with any provider (OpenAI, OpenRouter, Deepseek, Kimi, MiniMax, Ollama, CLIProxyAPI, etc.)
- 4 default bots seeded on first launch (Claude Sonnet 4, Claude Opus 4.6, Gemini 2.5 Pro, Gemini 3.1 Pro)
- 8 avatar color presets for visual differentiation

### Topic Management
- Create conversation topics with title and multiple bot participants
- Rename topics via double-click inline editing or right-click context menu
- Dynamically add/remove bots from topics via header popover or sidebar context menu
- Delete topics with confirmation dialog (cascades to messages and attachments)
- Topics sorted by most recent activity

### Messaging & Chat
- Send human messages; all topic bots respond sequentially with full conversation context
- **@mention specific bots** — type `@BotName` to target only certain bots, with keyboard-navigable autocomplete dropdown
- **Real-time SSE streaming** — bot responses stream token-by-token with live display
- **Group context injection** — system prompt tells each bot about other participants; other bots' messages prefixed with `[BotName]:`
- **Rate limit retry** — automatic exponential backoff on HTTP 429 (up to 3 retries, max 30s delay)

### File & Image Attachments
- Upload multiple files and images per message
- Vision-capable bots receive images as base64 data URLs
- Non-vision bots get text fallback `[Image attached: filename]`
- Text/code files injected as markdown code blocks (up to 15,000 chars with truncation)
- HTML files auto-converted to plain text

### URL Content Fetching
- Auto-detect URLs in human messages
- Fetch page content (HTML→text conversion) and inject as shared context for all bots
- Cached fetching — same URL fetched once per message, shared across all bots

### Topic Import/Export
- Export conversations as portable `.aigc.json` files with embedded base64 attachments
- Import conversations with automatic bot name matching to local bots
- Imported topics get "(imported)" title suffix with fresh UUIDs
- Attachments up to 10MB embedded; larger files skipped with annotation
- API keys and base URLs never included in exports (security)

### UI & UX
- **Resizable sidebar** (200–500px) with drag handle
- **Sidebar context menu** — right-click topics for Rename, Update Bots, Export, Delete
- **Markdown rendering** — GitHub Flavored Markdown with syntax highlighting (highlight.js)
- **Smart auto-scroll** — stays at bottom during streaming, preserves position when reading history
- **Streaming status indicators** — per-bot thinking/streaming/retry/error states

### Architecture
- **Tauri v2** desktop framework (Rust backend + React frontend)
- **SQLite** local database with WAL mode and foreign key constraints
- **Zustand** state management with real-time Tauri event integration
- **Separation of concerns** — testable `db_*()` functions separated from Tauri command wrappers
- **Database lock management** — released before async bot tasks (deadlock prevention)

### Testing
- 79 Rust unit/integration tests covering all backend modules
- 56 frontend tests covering components and state management
- CI pipeline with 3-platform Rust tests, frontend tests, and clippy/rustfmt linting

### Platforms
- macOS (Apple Silicon + Intel)
- Linux (Debian .deb + universal AppImage)
- Windows (NSIS .exe + MSI installer)
- Android (APK)
