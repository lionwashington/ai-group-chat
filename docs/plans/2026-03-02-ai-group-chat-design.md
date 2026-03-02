# AI Group Chat - Design Document

## Problem Statement

When researching a topic, switching between different LLMs (Claude, Gemini, GPT) is painful:
- No shared context between models
- Manual copy-paste of conversation history
- Cannot get multiple AI perspectives on the same topic simultaneously
- Different LLMs may catch different errors, but leveraging this requires manual effort

## Solution

A web-based group chat tool where humans and AI bots coexist. The core feature is **group discussions** where multiple AI models participate simultaneously, share context, and naturally form different perspectives on the same topic.

## Product Form

- **Web application** (Next.js full-stack)
- **Target users**: Personal / small team use
- **No SaaS features** needed (no billing, multi-tenancy)

## Architecture

### Approach: Monolithic Next.js + SSE

Single Next.js application handling both frontend and backend. Server-Sent Events (SSE) for streaming AI responses. SQLite for data storage.

**Why this approach:**
- Simplest to develop and deploy
- SSE is sufficient for AI streaming (same as ChatGPT)
- SQLite is zero-config, sufficient for personal/small team use
- Multiple Bot parallel replies handled via multiple SSE connections

### Tech Stack

| Component | Technology |
|-----------|-----------|
| Framework | Next.js App Router |
| Database | SQLite + Prisma ORM |
| Auth | NextAuth.js (email + password) |
| UI | shadcn/ui + Tailwind CSS |
| AI Integration | Vercel AI SDK |
| State | React hooks + SWR |

## Data Model

### User
- id, email, password_hash, name, avatar
- type: "human" | "bot"
- bot_config (bot only): { provider, model, system_prompt }

### ApiKey
- user_id, provider, encrypted_key
- Each user can configure one key per provider

### Conversation
- id, title, type: "direct" | "group"
- created_by (only humans can create)
- created_at, updated_at

### ConversationMember
- conversation_id, user_id
- role: "owner" | "member"

### Message
- id, conversation_id, sender_id
- content (markdown), created_at
- parent_id (optional, for @reply tracking)

**Key decisions:**
- Bots are a special type of User — unified message model
- Same AI provider can have multiple bots (e.g., Claude Opus vs Claude Sonnet with different system prompts)
- Conversation model is unified, `type` field distinguishes direct vs group
- Information isolation is achieved naturally via `conversation_id`

## Core Interaction Flows

### Group Discussion (Primary Feature)

1. Human sends message → saved to DB
2. System identifies all Bot members in the group
3. Parallel requests to all Bot APIs with full conversation history as context
4. Each Bot's response streams back via independent SSE connection
5. Frontend displays multiple streams simultaneously
6. All responses saved to DB

**Context passing**: Each Bot receives the complete message history of the group, including other Bots' responses. This naturally enables cross-model debate.

### Interaction Modes (Hybrid)

- **Default**: All Bots in the group auto-reply to every human message
- **@mention**: `@Claude what do you think about Gemini's point?` → only Claude replies
- Bots see each other's replies in context, naturally forming different viewpoints

### Direct Messages

Human sends message → if recipient is Bot, call API → stream response. If recipient is human, store message only (real-time push deferred to future WebSocket implementation).

## AI Provider Abstraction

```typescript
interface AIProvider {
  id: string;                    // "anthropic" | "google" | "openai"
  name: string;
  models: ModelInfo[];
  chat(params: ChatParams): AsyncIterable<string>;
}
```

Adding a new provider (Deepseek, Mistral, local Ollama, etc.) requires only implementing this interface.

First version implements: Anthropic (Claude), Google (Gemini), OpenAI (GPT).

## API Key Management

- **BYOK (Bring Your Own Key)**: Users configure their own API keys
- **Platform keys**: Admin can configure global API keys
- **Priority**: User key > Platform key
- Keys stored encrypted in database

## Project Structure

```
ai-group-chat/
├── src/
│   ├── app/                    # Next.js App Router pages
│   │   ├── (auth)/             # Login/register pages
│   │   ├── (chat)/             # Chat main interface
│   │   └── api/                # API Routes
│   │       ├── auth/           # Auth API
│   │       ├── conversations/  # Conversation CRUD
│   │       ├── messages/       # Messages + SSE streams
│   │       └── bots/           # Bot management
│   ├── components/             # React components
│   │   ├── chat/               # Chat components
│   │   ├── sidebar/            # Sidebar
│   │   └── ui/                 # shadcn/ui components
│   ├── lib/
│   │   ├── ai/                 # AI Provider implementations
│   │   │   ├── provider.ts     # Abstract interface
│   │   │   ├── anthropic.ts    # Claude
│   │   │   ├── google.ts       # Gemini
│   │   │   └── openai.ts       # OpenAI
│   │   ├── db/                 # Prisma client & schema
│   │   └── auth/               # Auth logic
│   └── hooks/                  # React hooks
├── prisma/
│   └── schema.prisma           # Database schema
└── package.json
```

## UI Layout

```
┌──────────────────────────────────────────────────────────┐
│  AI Group Chat                         [User Avatar] [⚙] │
├──────────────┬───────────────────────────────────────────┤
│              │                                           │
│  Conversations│  📌 Topic Title                           │
│              │  ─────────────────────────────────────── │
│  Direct      │                                           │
│  ├ Claude    │  [You] Compare React and Vue              │
│  ├ Gemini    │                                           │
│  └ GPT       │  [Claude] React's advantage is...         │
│              │  ████████░░ (streaming)                   │
│  Groups      │                                           │
│  ├ React讨论 │  [Gemini] From engineering perspective... │
│  └ 架构评审   │  ██████████ ✓                             │
│              │                                           │
│  [+ Chat]    │  ┌───────────────────────────────────┐   │
│  [+ Group]   │  │ @Claude your thoughts?     [Send] │   │
│              │  └───────────────────────────────────┘   │
└──────────────┴───────────────────────────────────────────┘
```

- Left sidebar: conversation list, separated into Direct and Groups
- Right: chat area with simultaneous streaming from multiple Bots
- Each message shows sender avatar/name, Bots have special badge
- Input supports @mention autocomplete
- Group settings to manage members (add/remove Bots)

## MVP Scope

### Included in v1:
- User registration/login (email + password)
- Bot management: create/edit/delete (provider + model + system prompt)
- API Key management: user configures own keys (encrypted storage)
- Direct messages: private chat with Bot or human
- Group creation: set title, invite multiple Bots
- Group chat: all Bots reply in parallel with streaming
- @mention: target specific Bot for reply
- Markdown rendering with code highlighting

### Deferred to future versions:
- Real-time push between humans (requires WebSocket)
- File/image upload
- Conversation export
- Autonomous Bot-to-Bot debate (without human trigger)
- Multi-language support
- Mobile responsive design
