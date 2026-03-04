# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.1.0] - 2026-03-04

### Added

- **Bot Management** — Create, edit, delete bots with name, avatar color, base URL, API key, model, system prompt, and vision support
- **Topic Management** — Create topics with multiple bots, rename (double-click), delete with confirmation
- **Dynamic Bot Assignment** — Add/remove bots from topics via header popover or sidebar context menu
- **Messaging** — Send human messages, @mention specific bots for targeted replies
- **AI Chat** — Sequential bot processing with SSE streaming, rate limit retry with exponential backoff
- **Group Context** — System prompt tells each bot about other participants; other bots' messages prefixed with `[BotName]:`
- **Attachments** — Upload files and images; vision bots receive base64 image data, non-vision bots get text fallback
- **URL Content Fetching** — Auto-detect URLs in messages, fetch page content (HTML to text), share across bots
- **Topic Import/Export** — Export as `.aigc.json` with embedded attachments; import with bot name matching
- **Markdown Rendering** — GFM tables, syntax highlighting via highlight.js
- **Smart Auto-scroll** — Stays at bottom during streaming, preserves position when reading history
- **@mention Autocomplete** — Keyboard-navigable dropdown with Arrow keys, Enter/Tab, Escape
- **Resizable Sidebar** — Drag handle (200-500px)
- **Sidebar Context Menu** — Right-click topics for Rename, Update Bots, Export, Delete
- **4 Default Bots** — Claude Sonnet 4, Claude Opus 4.6, Gemini 2.5 Pro, Gemini 3.1 Pro (seeded on first launch)
- **79 Rust tests** and **56 frontend tests** — comprehensive coverage across all modules
