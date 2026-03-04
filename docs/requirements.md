# AI Group Chat - Requirements Document

> Last updated: 2026-03-04

## 1. Product Overview

AI Group Chat is a desktop application that enables humans to chat with multiple AI bots simultaneously in a shared conversation context. Unlike 1-on-1 AI chat tools, this app lets multiple AI models (Claude, Gemini, GPT, etc.) debate the same topic, see each other's responses, and build on the conversation collectively.

**Target Users**: Developers, researchers, and AI enthusiasts who want to compare model outputs or leverage multi-model collaboration.

**Platform**: macOS / Windows / Linux (via Tauri v2)

## 2. Core Features

### 2.1 Bot Management

| ID | Requirement | Status |
|----|-------------|--------|
| REQ-BOT-01 | Create bots with name, avatar color, base URL, API key, model, system prompt, and vision support flag | Done |
| REQ-BOT-02 | Edit any bot field after creation | Done |
| REQ-BOT-03 | Delete bots (cascade removes from topic associations, messages keep sender_bot_id=NULL) | Done |
| REQ-BOT-04 | List all bots ordered by creation date | Done |
| REQ-BOT-05 | Seed 4 default bots on first launch (Claude Sonnet 4, Claude Opus 4.6, Gemini 2.5 Pro, Gemini 3.1 Pro) | Done |
| REQ-BOT-06 | All bots use OpenAI-compatible `/chat/completions` endpoint (no provider-specific code) | Done |
| REQ-BOT-07 | 8 avatar color presets for visual differentiation | Done |

### 2.2 Topic Management

| ID | Requirement | Status |
|----|-------------|--------|
| REQ-TOPIC-01 | Create topics with a title and one or more associated bots | Done |
| REQ-TOPIC-02 | List topics ordered by last activity (updated_at DESC) | Done |
| REQ-TOPIC-03 | Show topic summary: title, bot count, last message preview | Done |
| REQ-TOPIC-04 | Delete topics with confirmation dialog (cascades to all messages and attachments) | Done |
| REQ-TOPIC-05 | Update topic's bot associations (add/remove bots from active topic) | Done |

### 2.3 Messaging

| ID | Requirement | Status |
|----|-------------|--------|
| REQ-MSG-01 | Send human text messages to a topic | Done |
| REQ-MSG-02 | @mention specific bots to have only those bots respond | Done |
| REQ-MSG-03 | Messages ordered chronologically (created_at ASC) | Done |
| REQ-MSG-04 | Markdown rendering with GFM tables, syntax highlighting (highlight.js) | Done |
| REQ-MSG-05 | Visual distinction: human messages right-aligned (primary), bot messages left-aligned (muted) with colored avatar | Done |

### 2.4 AI Chat

| ID | Requirement | Status |
|----|-------------|--------|
| REQ-CHAT-01 | Bots process sequentially (not parallel) so each bot sees all prior responses in the same round | Done |
| REQ-CHAT-02 | SSE streaming: real-time token-by-token display via Tauri events | Done |
| REQ-CHAT-03 | "Thinking" status indicator with bouncing dots before first token | Done |
| REQ-CHAT-04 | Rate limit handling: automatic retry with exponential backoff (max 3 retries), amber retry info in UI | Done |
| REQ-CHAT-05 | Group chat context: system prompt tells each bot about other participants, prefixes other bots' messages with `[BotName]:` | Done |
| REQ-CHAT-06 | System prompt injection per bot (configurable per bot) | Done |

### 2.5 Attachments

| ID | Requirement | Status |
|----|-------------|--------|
| REQ-ATT-01 | Upload files (any type) and images to messages | Done |
| REQ-ATT-02 | Image attachments: vision-capable bots receive base64 data URLs; non-vision bots get text fallback `[Image: filename]` | Done |
| REQ-ATT-03 | File attachments: content read and truncated to 15,000 chars, included as code block in context | Done |
| REQ-ATT-04 | Files stored on disk in `{app_data}/attachments/` with sanitized names | Done |
| REQ-ATT-05 | File upload preview with remove button before sending | Done |

### 2.6 URL Content Fetching

| ID | Requirement | Status |
|----|-------------|--------|
| REQ-URL-01 | Auto-detect HTTP/HTTPS URLs in human messages | Done |
| REQ-URL-02 | Fetch URL content (HTML to text via html2text), truncate to 5,000 chars | Done |
| REQ-URL-03 | URL cache: fetch once per round, shared across all bots | Done |
| REQ-URL-04 | Appended to human messages only, not injected into bot messages | Done |
| REQ-URL-05 | 10-second timeout per URL, silent failure on error | Done |

### 2.7 Topic Import/Export

| ID | Requirement | Status |
|----|-------------|--------|
| REQ-TRANSFER-01 | Export topic as `.aigc.json` file containing title, bots metadata, messages, and base64-encoded attachments | Done |
| REQ-TRANSFER-02 | No sensitive data in export (api_key, base_url omitted) | Done |
| REQ-TRANSFER-03 | Attachments >10MB skipped with reason; missing files handled gracefully | Done |
| REQ-TRANSFER-04 | Import creates new topic with "(imported)" suffix | Done |
| REQ-TRANSFER-05 | Bots matched by name to local bots; unmatched bots result in NULL sender_bot_id | Done |
| REQ-TRANSFER-06 | Attachments decoded from base64 and written to disk | Done |
| REQ-TRANSFER-07 | Format and version validation on import | Done |
| REQ-TRANSFER-08 | Native OS save/open file dialogs for export/import | Done |

### 2.8 UI/UX

| ID | Requirement | Status |
|----|-------------|--------|
| REQ-UI-01 | Resizable sidebar (200-500px drag handle) | Done |
| REQ-UI-02 | Smart auto-scroll: disabled on scroll-up, re-enabled on scroll-to-bottom or new send | Done |
| REQ-UI-03 | @mention autocomplete dropdown with keyboard navigation (Arrow keys, Enter/Tab, Escape) | Done |
| REQ-UI-04 | Auto-resizing textarea (40-120px height) | Done |
| REQ-UI-05 | Bot badges in topic header showing participant names with avatar colors | Done |
| REQ-UI-06 | Empty state when no topic selected | Done |

## 3. Non-Functional Requirements

| ID | Requirement | Status |
|----|-------------|--------|
| REQ-NF-01 | All HTTP requests from Rust backend (no CORS issues) | Done |
| REQ-NF-02 | SQLite with WAL mode for concurrent reads | Done |
| REQ-NF-03 | DB lock released before spawning async bot tasks (deadlock prevention) | Done |
| REQ-NF-04 | Local-first: all data stored locally, no cloud dependency | Done |
| REQ-NF-05 | Universal AI provider support via OpenAI-compatible API | Done |

## 4. Export JSON Schema

File extension: `.aigc.json`

```json
{
  "format": "ai-group-chat-export",
  "version": 1,
  "exported_at": "2026-03-04T12:00:00Z",
  "topic": {
    "title": "...",
    "created_at": "..."
  },
  "bots": [
    {
      "name": "...",
      "avatar_color": "...",
      "model": "...",
      "system_prompt": "...",
      "supports_vision": true
    }
  ],
  "messages": [
    {
      "sender_type": "human|bot",
      "sender_bot_name": "Claude Opus 4.6" | null,
      "content": "...",
      "created_at": "...",
      "attachments": [
        {
          "file_name": "...",
          "file_type": "image|file",
          "mime_type": "...",
          "data_base64": "..." | null,
          "skipped": false,
          "skip_reason": null
        }
      ]
    }
  ]
}
```

## 5. Future Considerations

- Per-message token count tracking
- Conversation branching / forking
- Bot response ordering configuration (random vs fixed)
- Export to other formats (Markdown, HTML)
- Plugin system for custom AI providers
- Shared topics via network sync
