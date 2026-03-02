# AI Group Chat - Test Cases

## Unit Tests (Rust Backend)

### UT-DB: Database Schema & Migrations

| ID | Test Case | Expected |
|----|-----------|----------|
| UT-DB-01 | Run migrations on fresh DB | All tables created, no errors |
| UT-DB-02 | Run migrations twice (idempotent) | No errors on second run |
| UT-DB-03 | Foreign key constraints enabled | Inserting message with invalid topic_id fails |

### UT-BOT: Bot CRUD

| ID | Test Case | Expected |
|----|-----------|----------|
| UT-BOT-01 | Create bot with all fields | Bot saved, returned with generated ID |
| UT-BOT-02 | Create bot with only required fields | Defaults applied (avatar_color, empty api_key, etc.) |
| UT-BOT-03 | List bots when empty | Returns empty vec |
| UT-BOT-04 | List bots after creating 3 | Returns 3 bots, ordered by created_at |
| UT-BOT-05 | Update bot name only | Name updated, other fields unchanged |
| UT-BOT-06 | Update bot all fields | All fields updated |
| UT-BOT-07 | Delete bot | Bot no longer in list |
| UT-BOT-08 | Delete non-existent bot | No error (idempotent) |
| UT-BOT-09 | Create bot with empty name | Should fail validation |
| UT-BOT-10 | Create bot with empty base_url | Should fail validation |

### UT-TOPIC: Topic CRUD

| ID | Test Case | Expected |
|----|-----------|----------|
| UT-TOPIC-01 | Create topic with title and 2 bot_ids | Topic created, both bots associated |
| UT-TOPIC-02 | Create topic with no bots | Topic created with empty bot list |
| UT-TOPIC-03 | List topics when empty | Returns empty vec |
| UT-TOPIC-04 | List topics ordered by updated_at DESC | Most recent first |
| UT-TOPIC-05 | Get topic includes associated bots | Returned Topic has correct bots vec |
| UT-TOPIC-06 | Update topic bots (replace all) | Old associations removed, new ones added |
| UT-TOPIC-07 | Delete topic cascades to topic_bots | topic_bots entries removed |
| UT-TOPIC-08 | Delete topic cascades to messages | Messages for that topic removed |
| UT-TOPIC-09 | Topic summary includes last_message_preview | After adding messages, preview shows latest |
| UT-TOPIC-10 | Topic summary bot_count is accurate | Reflects actual associated bot count |

### UT-MSG: Message Storage

| ID | Test Case | Expected |
|----|-----------|----------|
| UT-MSG-01 | Save human message | Stored with sender_type="human", sender_bot_id=NULL |
| UT-MSG-02 | Save bot message | Stored with sender_type="bot", correct sender_bot_id |
| UT-MSG-03 | List messages for topic, ordered by created_at ASC | Chronological order |
| UT-MSG-04 | List messages returns empty for new topic | Empty vec |
| UT-MSG-05 | Messages isolated between topics | Topic A messages not in Topic B query |
| UT-MSG-06 | Save message updates topic updated_at | Topic's updated_at changes |

### UT-ATT: Attachment Handling

| ID | Test Case | Expected |
|----|-----------|----------|
| UT-ATT-01 | Save image attachment | File written to disk, DB record with file_type="image" |
| UT-ATT-02 | Save file attachment | File written to disk, DB record with file_type="file" |
| UT-ATT-03 | Read attachment as base64 | Returns correct base64 encoding |
| UT-ATT-04 | List messages includes attachments | Attachments nested in message response |
| UT-ATT-05 | Delete message cascades to attachments | Attachment DB records removed |
| UT-ATT-06 | Attachment file_type determined by mime_type | image/png → "image", text/plain → "file" |

### UT-AI: AI Client & Stream Parser

| ID | Test Case | Expected |
|----|-----------|----------|
| UT-AI-01 | Build ChatRequest with system prompt | System message is first in messages array |
| UT-AI-02 | Build ChatRequest without system prompt | No system message in array |
| UT-AI-03 | Build messages from mixed human/bot history | Correct role mapping: human→user, bot→assistant |
| UT-AI-04 | Build messages with image attachment (vision bot) | image_url content part with base64 data |
| UT-AI-05 | Build messages with image attachment (non-vision bot) | Text fallback: "[Image attached: file.png]" |
| UT-AI-06 | Build messages with file attachment | File content injected as text block |
| UT-AI-07 | Parse SSE stream: single delta | on_delta called once with correct text |
| UT-AI-08 | Parse SSE stream: multiple deltas | on_delta called for each, full_content accumulated |
| UT-AI-09 | Parse SSE stream: [DONE] signal | Stream ends, returns full content |
| UT-AI-10 | Parse SSE stream: empty lines and comments | Skipped, no errors |
| UT-AI-11 | Parse SSE stream: malformed JSON line | Skipped gracefully, no crash |
| UT-AI-12 | Parse SSE stream: chunked data (partial lines across chunks) | Buffer handles correctly, no lost data |
| UT-AI-13 | ChatRequest serializes to valid OpenAI-compatible JSON | model, messages, stream fields present |
| UT-AI-14 | Authorization header set when api_key present | Bearer token in request |
| UT-AI-15 | No Authorization header when api_key empty | Header omitted |

---

## Unit Tests (React Frontend)

### UT-STORE: Zustand Store

| ID | Test Case | Expected |
|----|-----------|----------|
| UT-STORE-01 | setBots replaces bot list | bots state updated |
| UT-STORE-02 | addBot appends to list | New bot at end of list |
| UT-STORE-03 | removeBot filters by id | Bot no longer in list |
| UT-STORE-04 | updateBotInStore replaces matching bot | Updated bot in place, others unchanged |
| UT-STORE-05 | setActiveTopicId updates state | activeTopicId changed |
| UT-STORE-06 | handleStreamEvent accumulates delta content | content grows with each delta |
| UT-STORE-07 | handleStreamEvent ignores wrong topic_id | streamingStates unchanged |
| UT-STORE-08 | handleStreamEvent sets done=true on completion | Streaming state marked done |
| UT-STORE-09 | handleStreamEvent stores error | error field populated |
| UT-STORE-10 | isAnyBotStreaming returns true during streaming | At least one bot not done |
| UT-STORE-11 | isAnyBotStreaming returns false when all done | All bots done |
| UT-STORE-12 | clearStreaming resets all streaming states | streamingStates empty |

### UT-COMP: React Components

| ID | Test Case | Expected |
|----|-----------|----------|
| UT-COMP-01 | Sidebar renders topic list | Topics displayed in order |
| UT-COMP-02 | Sidebar highlights active topic | Active topic has accent background |
| UT-COMP-03 | MessageBubble renders human message (right-aligned) | "You" avatar on right, primary background |
| UT-COMP-04 | MessageBubble renders bot message (left-aligned) | Bot avatar on left, muted background, bot name shown |
| UT-COMP-05 | MessageBubble renders markdown content | Headers, code blocks, lists rendered |
| UT-COMP-06 | MessageBubble shows attachment indicators | File/image icons with filenames |
| UT-COMP-07 | StreamingMessage shows cursor animation while streaming | Pulsing cursor visible when done=false |
| UT-COMP-08 | StreamingMessage shows error message | Red error text displayed |
| UT-COMP-09 | StreamingMessage removes cursor when done | No cursor when done=true |
| UT-COMP-10 | MessageInput sends on Enter (not Shift+Enter) | onSend called on Enter, newline on Shift+Enter |
| UT-COMP-11 | MessageInput shows @mention dropdown on @ | Dropdown appears with bot list |
| UT-COMP-12 | MessageInput filters @mention list by typed text | Only matching bots shown |
| UT-COMP-13 | MessageInput inserts bot name on @mention select | Content updated with @BotName |
| UT-COMP-14 | MessageInput shows file previews | Attached files listed with remove button |
| UT-COMP-15 | MessageInput disabled during streaming | Input and send button disabled |
| UT-COMP-16 | BotCard displays bot info | Name, model, avatar color, vision badge |
| UT-COMP-17 | BotFormDialog pre-fills fields when editing | All fields populated from editBot |
| UT-COMP-18 | BotFormDialog empty fields when creating | All fields blank/defaults |
| UT-COMP-19 | CreateTopicDialog shows bot checkboxes | All bots listed with checkboxes |
| UT-COMP-20 | CreateTopicDialog disables submit without title or bots | Button disabled |

---

## Integration Tests

### IT-FLOW: End-to-End Data Flows (Rust, with test DB)

| ID | Test Case | Expected |
|----|-----------|----------|
| IT-FLOW-01 | Create bot → create topic with bot → verify topic has bot | Full flow succeeds |
| IT-FLOW-02 | Create topic → send message → list messages | Message appears in topic |
| IT-FLOW-03 | Send message with attachment → list messages | Attachment nested in message |
| IT-FLOW-04 | Delete bot used in topic → topic still exists | Bot removed from topic_bots, topic intact |
| IT-FLOW-05 | Delete topic → messages and attachments cleaned up | CASCADE delete works |
| IT-FLOW-06 | Multiple topics → messages isolated | Topic A's messages not in Topic B |

### IT-AI: AI Client Integration (with mock HTTP server)

| ID | Test Case | Expected |
|----|-----------|----------|
| IT-AI-01 | Send chat request to mock server → receive streamed response | Full content assembled from stream |
| IT-AI-02 | Mock server returns error 401 | Error propagated with status and body |
| IT-AI-03 | Mock server returns error 429 (rate limit) | Error message includes rate limit info |
| IT-AI-04 | Mock server drops connection mid-stream | Error handled, partial content preserved |
| IT-AI-05 | Two parallel bot requests to mock server | Both complete independently |
| IT-AI-06 | Chat request includes image content part | Mock server receives valid multimodal request |

### IT-CHAT: Chat Flow Integration (Rust, with mock HTTP)

| ID | Test Case | Expected |
|----|-----------|----------|
| IT-CHAT-01 | chat_with_bots: 1 bot, no @mention | Bot receives full history, emits stream events, saves message |
| IT-CHAT-02 | chat_with_bots: 3 bots, no @mention | All 3 bots respond in parallel, 3 messages saved |
| IT-CHAT-03 | chat_with_bots: 3 bots, @mention 1 | Only mentioned bot responds, 1 message saved |
| IT-CHAT-04 | chat_with_bots: bot API fails | Error event emitted, other bots unaffected |
| IT-CHAT-05 | chat_with_bots: context includes previous bot messages | Second round of chat includes first round responses |
| IT-CHAT-06 | chat_with_bots: message with image, vision bot | Image sent as base64 content part |
| IT-CHAT-07 | chat_with_bots: message with image, non-vision bot | Image sent as text fallback |

### IT-UI: Frontend Integration (React Testing Library, mocked Tauri)

| ID | Test Case | Expected |
|----|-----------|----------|
| IT-UI-01 | App loads → fetches bots and topics | listBots and listTopics called |
| IT-UI-02 | Click topic → loads messages | getTopic and listMessages called, messages rendered |
| IT-UI-03 | Create bot via dialog → appears in list | createBot called, bot added to store |
| IT-UI-04 | Create topic via dialog → appears in sidebar | createTopic called, topic in sidebar |
| IT-UI-05 | Send message → bot streams response | sendHumanMessage called, stream events update UI |
| IT-UI-06 | Multiple bots streaming → all visible simultaneously | Multiple StreamingMessage components rendered |
| IT-UI-07 | Stream completes → messages reloaded from DB | listMessages called after all bots done |
| IT-UI-08 | Upload file → preview shown → sent with message | File in attachment preview, saveAttachment called |

---

## E2E Tests (Deferred to post-MVP)

### E2E: Full Application (Tauri WebDriver)

| ID | Test Case | Expected |
|----|-----------|----------|
| E2E-01 | Fresh app launch → empty state | No topics, prompt to create |
| E2E-02 | Add bot → create topic → send message → receive AI response | Full happy path |
| E2E-03 | Multi-bot topic → all bots respond | Multiple responses visible |
| E2E-04 | @mention specific bot → only that bot responds | Single response |
| E2E-05 | Upload image → bot analyzes it | Image sent, response references image content |
| E2E-06 | Delete topic → removed from sidebar | Topic gone, chat view cleared |
| E2E-07 | Edit bot config → next response uses new config | Updated model/prompt reflected |
| E2E-08 | App restart → data persists | Topics, messages, bots all retained |

---

## Test Infrastructure

### Rust Tests
- **Framework**: built-in `#[cfg(test)]` + `#[test]`
- **DB**: each test creates an in-memory SQLite DB (`Connection::open_in_memory()`)
- **HTTP mocks**: `mockito` crate for mock AI API server
- **Async**: `#[tokio::test]` for async tests

### React Tests
- **Framework**: Vitest + React Testing Library
- **Tauri mock**: mock `@tauri-apps/api/core` `invoke()` function
- **Event mock**: mock `@tauri-apps/api/event` `listen()` function

### Test Commands
```bash
# Rust unit + integration tests
cd src-tauri && cargo test

# React unit + integration tests
npm run test

# All tests
npm run test:all
```

---

## Summary

| Layer | Count | Status |
|-------|-------|--------|
| Rust Unit Tests | 37 | To implement |
| React Unit Tests (Store) | 12 | To implement |
| React Unit Tests (Components) | 20 | To implement |
| Integration Tests (Rust) | 13 | To implement |
| Integration Tests (React) | 8 | To implement |
| E2E Tests | 8 | Deferred to post-MVP |
| **Total (MVP)** | **90** | |
| **Total (with E2E)** | **98** | |
