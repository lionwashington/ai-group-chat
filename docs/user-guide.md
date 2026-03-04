# AI Group Chat - User Guide

## Getting Started

### Installation

Download the installer for your platform from the [Releases page](https://github.com/lionwashington/ai-group-chat/releases):

| Platform | File |
|----------|------|
| macOS (Apple Silicon) | `ai-group-chat_*_aarch64.dmg` |
| macOS (Intel) | `ai-group-chat_*_x64.dmg` |
| Linux (Debian/Ubuntu) | `ai-group-chat_*_amd64.deb` |
| Linux (Universal) | `ai-group-chat_*.AppImage` |
| Windows | `ai-group-chat_*_x64-setup.exe` or `.msi` |

### First Launch

When you open the app for the first time, it creates a local SQLite database with 4 default bots (Claude Sonnet 4, Claude Opus 4.6, Gemini 2.5 Pro, Gemini 3.1 Pro). You'll need to configure at least one bot with your API credentials before chatting.

## Managing Bots

Click **Manage Bots** in the sidebar to open the bot manager.

### Add a Bot

Click **Add Bot** and fill in the fields:

| Field | Required | Description |
|-------|----------|-------------|
| Name | Yes | Display name shown in chat (e.g. "Claude Opus") |
| Base URL | Yes | API endpoint — must be OpenAI-compatible (see [Supported Providers](#supported-providers)) |
| API Key | Yes | Your API key for that provider |
| Model | Yes | Model identifier (e.g. `claude-sonnet-4-5`, `gpt-4o`, `gemini-2.5-pro`) |
| System Prompt | No | Custom instructions for the bot's behavior |
| Avatar Color | No | Pick from 8 color presets for visual distinction |
| Supports Vision | No | Check this if the model can process images |

### Edit / Delete a Bot

In the bot manager, click the **pencil icon** to edit or the **trash icon** to delete a bot. Deleting a bot removes it from all topics, but existing messages from that bot are preserved.

### Supported Providers

Any service that implements the OpenAI chat completions API works. Set the **Base URL** to:

| Provider | Base URL |
|----------|----------|
| OpenAI | `https://api.openai.com/v1` |
| OpenRouter | `https://openrouter.ai/api/v1` |
| Deepseek | `https://api.deepseek.com/v1` |
| Kimi (Moonshot) | `https://api.moonshot.cn/v1` |
| MiniMax | `https://api.minimax.chat/v1` |
| Ollama (local) | `http://localhost:11434/v1` |
| CLIProxyAPI | `http://localhost:8080/v1` |

## Working with Topics

A **topic** is a conversation thread. Each topic has a title and one or more bots.

### Create a Topic

1. Click **+ New Topic** in the sidebar
2. Enter a title (e.g. "React vs Vue debate")
3. Select one or more bots to participate
4. Click **Create**

### Switch Topics

Click any topic in the sidebar to switch to it. Topics are sorted by most recent activity.

### Rename a Topic

Two ways:
- **Double-click** the topic title in the sidebar and edit inline (press Enter to save, Escape to cancel)
- **Right-click** the topic → **Rename**

### Edit Topic Bots

To add or remove bots from an existing topic:
- Click the **pencil icon** (Edit Bots) in the chat header
- Or **right-click** the topic in the sidebar → **Update Bots**

A popover appears with checkboxes for all available bots. Toggle the bots you want, then click **Save**.

### Delete a Topic

- Hover over a topic in the sidebar and click the **X** button
- Or **right-click** → **Delete**

A confirmation dialog will appear. Deleting a topic removes all its messages and attachments.

## Chatting

### Send a Message

Type your message in the input box and press **Enter** to send. Use **Shift+Enter** for a new line.

After you send a message, each bot in the topic responds **sequentially** — one at a time, streaming in real-time. Each bot sees the full conversation history, including the other bots' responses.

### @Mention Specific Bots

By default, all bots in the topic respond to your message. To target specific bots:

1. Type `@` in the message input
2. A dropdown appears listing the topic's bots
3. Select a bot (click, press Enter, or press Tab)
4. Only the mentioned bot(s) will respond

Example: `@Claude What do you think about Gemini's analysis?`

### Attach Files & Images

Click the **paperclip icon** to attach files to your message:

- **Images** (PNG, JPG, etc.) are sent as visual content to vision-capable bots
- **Text files** (code, documents, etc.) are injected as text context in the message
- Multiple files can be attached to a single message
- Attached files appear as thumbnails above the input box — click the X to remove one

### URL Content Fetching

Paste a URL in your message text (e.g. `https://example.com/article`) and the app will fetch the page content and inject it as context for all bots.

### Auto-Scroll

The chat automatically scrolls to the latest message while you're near the bottom. If you scroll up to read earlier messages, auto-scroll pauses so you won't lose your place.

### Markdown Rendering

Bot responses are rendered as Markdown with:
- **GitHub Flavored Markdown** (tables, task lists, strikethrough)
- **Syntax highlighting** for code blocks
- Inline code, bold, italic, links, etc.

## Import & Export

### Export a Topic

Save a conversation as a portable `.aigc.json` file:

1. Click the **download icon** in the chat header
2. Or **right-click** the topic in the sidebar → **Export Topic**
3. Choose a save location in the file dialog

The export includes: topic title, all messages, bot metadata (name, model, system prompt, avatar color), and embedded file attachments (base64-encoded, up to 10MB each).

**Note:** API keys and base URLs are **never** included in exports for security.

### Import a Topic

Load a previously exported conversation:

1. Click **Import** in the sidebar (upload icon)
2. Select a `.aigc.json` file
3. A new topic is created with "(imported)" appended to the title

During import:
- Bots are matched to your local bots by name
- If a matching bot is found, it's automatically linked to the imported topic
- If no match is found, messages from that bot are preserved but unlinked
- Attachments are decoded and stored locally

## Keyboard Shortcuts

| Action | Shortcut |
|--------|----------|
| Send message | `Enter` |
| New line in message | `Shift+Enter` |
| @mention autocomplete | Type `@` then arrow keys + Enter/Tab |
| Cancel rename | `Escape` |
| Confirm rename | `Enter` |

## Data Storage

All data is stored locally on your machine — no cloud accounts or servers required.

| Platform | Database Location |
|----------|-------------------|
| macOS | `~/Library/Application Support/com.aigroupchat.desktop/` |
| Linux | `~/.local/share/com.aigroupchat.desktop/` |
| Windows | `%APPDATA%\com.aigroupchat.desktop\` |

The database file is `ai-group-chat.db` (SQLite). File attachments are stored in the `attachments/` subdirectory.

## Troubleshooting

### Bot not responding

- Verify the **Base URL** and **API Key** are correct in Manage Bots
- Check that the **Model** identifier is valid for your provider
- Ensure your API key has sufficient credits/quota

### Messages show error

If a bot fails to respond, an error message will appear in the chat. Common causes:
- Network connectivity issues
- Invalid API credentials
- Rate limiting by the provider
- Model not available (wrong model name or access not granted)

### Data migration after update

If the app's bundle identifier changes between versions, your data may appear to be missing. The database is stored in a directory named after the bundle identifier. Copy the `ai-group-chat.db` file and `attachments/` folder from the old directory to the new one.
