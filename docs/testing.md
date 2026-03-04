# AI Group Chat - Testing Document

> Last updated: 2026-03-04

## 1. Overview

| Layer | Framework | Tests | Status |
|-------|-----------|-------|--------|
| Rust unit tests | built-in `#[test]` | 79 | All passing |
| Frontend unit tests | Vitest + React Testing Library | 56 | All passing |
| E2E tests | WebdriverIO | 8 | Deferred |

**Total: 135 automated tests** (79 Rust + 56 Frontend)

## 2. Running Tests

```bash
# Rust tests (from project root)
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo test --manifest-path src-tauri/Cargo.toml

# Frontend tests
npm test              # single run
npm run test:watch    # watch mode
npm run test:ui       # Vitest UI

# E2E tests (requires running dev server)
npm run test:e2e

# All unit/integration tests
npm run test:all
```

## 3. Test Infrastructure

### 3.1 Rust Test Pattern

Every command module uses the same test setup:

```rust
fn setup_test_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    crate::db::schema::run_migrations(&conn).unwrap();
    conn
}
```

- In-memory SQLite (fast, no cleanup)
- Foreign keys enabled (matches production)
- Full schema migrations applied
- Helper functions: `create_test_bot()`, `create_test_topic()`, `create_test_message()`, `temp_attachments_dir()`

### 3.2 Frontend Test Setup

**`src/test/setup.ts`**:
- Polyfills `ResizeObserver` (not in jsdom)
- Mocks `@tauri-apps/api/core` `invoke()` (returns empty arrays)
- Mocks `@tauri-apps/api/event` `listen()` (no-op)

**`src/test/fixtures.ts`** — Factory functions:
- `makeBotFixture(overrides?)` — returns `Bot` with defaults
- `makeMessageFixture(overrides?)` — returns `Message` with defaults
- `makeTopicSummaryFixture(overrides?)` — returns `TopicSummary` with defaults
- `makeStreamingStateFixture(overrides?)` — returns `StreamingState` with defaults
- `makeAttachmentFixture(overrides?)` — returns `Attachment` with defaults

**Environment**: Vitest with jsdom, globals enabled, path alias `@/*`

## 4. Rust Test Matrix (76 tests)

### 4.1 Database Schema — 3 tests

| ID | Test | Validates |
|----|------|-----------|
| UT-DB-01 | `test_migrations_fresh_db` | All tables created on fresh DB |
| UT-DB-02 | `test_migrations_idempotent` | Running migrations twice is safe |
| UT-DB-03 | `test_foreign_key_constraints` | FK enforcement (insert with invalid reference fails) |

### 4.2 Bot Commands — 10 tests

| ID | Test | Validates |
|----|------|-----------|
| UT-BOT-01 | `test_create_bot_all_fields` | Create bot with all fields populated |
| UT-BOT-02 | `test_create_bot_defaults` | Default avatar_color, empty api_key/system_prompt |
| UT-BOT-03 | `test_list_bots_empty` | Empty list on fresh DB |
| UT-BOT-04 | `test_list_bots_ordered` | Bots ordered by created_at |
| UT-BOT-05 | `test_update_bot_name_only` | Partial update preserves other fields |
| UT-BOT-06 | `test_update_bot_all_fields` | Update all fields simultaneously |
| UT-BOT-07 | `test_delete_bot` | Bot removed from DB |
| UT-BOT-08 | `test_delete_nonexistent_bot` | Idempotent delete (no error) |
| UT-BOT-09 | `test_create_bot_empty_name_fails` | Validation: name required |
| UT-BOT-10 | `test_create_bot_empty_base_url_fails` | Validation: base_url required |

### 4.3 Topic Commands — 13 tests

| ID | Test | Validates |
|----|------|-----------|
| UT-TOPIC-01 | `test_create_topic_with_bots` | Topic created with bot associations |
| UT-TOPIC-02 | `test_create_topic_no_bots` | Topic with empty bot list |
| UT-TOPIC-03 | `test_list_topics_empty` | Empty list on fresh DB |
| UT-TOPIC-04 | `test_list_topics_ordered_by_updated_at_desc` | Most recent first |
| UT-TOPIC-05 | `test_get_topic_includes_bots` | Nested bots in response |
| UT-TOPIC-06 | `test_update_topic_bots_replaces_all` | Replace (not append) bot list |
| UT-TOPIC-07 | `test_delete_topic_cascades_to_topic_bots` | CASCADE to join table |
| UT-TOPIC-08 | `test_delete_topic_cascades_to_messages` | CASCADE to messages |
| UT-TOPIC-09 | `test_topic_summary_last_message_preview` | Preview from latest message |
| UT-TOPIC-10 | `test_topic_summary_bot_count` | Correct bot count in summary |
| UT-TOPIC-11 | `test_rename_topic` | Title updated, updated_at refreshed |
| UT-TOPIC-12 | `test_rename_topic_trims_whitespace` | Whitespace trimmed from title |
| UT-TOPIC-13 | `test_rename_topic_empty_title_fails` | Empty/blank title rejected |

### 4.4 Message Commands — 6 tests

| ID | Test | Validates |
|----|------|-----------|
| UT-MSG-01 | `test_send_human_message` | Human message with sender_type="human" |
| UT-MSG-02 | `test_save_bot_message` | Bot message with sender_bot_id |
| UT-MSG-03 | `test_list_messages_ordered` | Chronological order (ASC) |
| UT-MSG-04 | `test_list_messages_empty` | Empty list for topic with no messages |
| UT-MSG-05 | `test_messages_isolated_between_topics` | Messages scoped to topic |
| UT-MSG-06 | `test_save_message_updates_topic_updated_at` | Topic timestamp refreshed |

### 4.5 Attachment Commands — 6 tests

| ID | Test | Validates |
|----|------|-----------|
| UT-ATT-01 | `test_save_image_attachment` | File written, DB record, file_type="image" |
| UT-ATT-02 | `test_save_file_attachment` | file_type="file" for non-image |
| UT-ATT-03 | `test_read_attachment_base64` | Correct base64 encoding |
| UT-ATT-04 | `test_list_messages_includes_attachments` | Nested attachments in messages |
| UT-ATT-05 | `test_delete_message_cascades_to_attachments` | CASCADE cleanup |
| UT-ATT-06 | `test_file_type_determined_by_mime_type` | image/* → "image", others → "file" |

### 4.6 Chat / AI — 12 tests

| ID | Test | Validates |
|----|------|-----------|
| UT-AI-01 | `test_build_messages_with_system_prompt` | System prompt as first message |
| UT-AI-02 | `test_build_messages_without_system_prompt` | No system message if empty prompt |
| UT-AI-03 | `test_build_messages_mixed_history` | Role mapping: human→user, own→assistant, other→user+prefix |
| UT-AI-04 | `test_build_messages_image_vision_bot` | Base64 image_url for vision bots |
| UT-AI-05 | `test_build_messages_image_non_vision_bot` | Text fallback `[Image: name]` |
| UT-AI-06 | `test_build_messages_file_attachment` | File content as code block |
| UT-AI-06b | `test_build_messages_mixed_attachments_vision` | Images + files combined |
| UT-AI-07 | `test_build_messages_with_url_cache` | URL content appended to human messages |
| UT-AI-08 | `test_build_messages_url_cache_not_applied_to_bot_messages` | Bot messages unaffected |
| UT-AI-09 | `test_build_messages_empty_url_cache` | Empty cache = no changes |
| UT-AI-13 | `test_chat_request_serializes_to_valid_json` | ChatRequest JSON format |
| UT-AI-13b | `test_chat_request_with_image_serializes` | Multimodal content serialization |

### 4.7 SSE Stream Parser — 8 tests

| ID | Test | Validates |
|----|------|-----------|
| UT-AI-07 | `test_sse_single_delta` | Single chunk parsed |
| UT-AI-08 | `test_sse_multiple_deltas` | Multiple chunks concatenated |
| UT-AI-09 | `test_sse_done_returns_full_content` | `[DONE]` marker handled |
| UT-AI-10 | `test_sse_empty_lines_and_comments_skipped` | Noise filtered |
| UT-AI-11 | `test_sse_malformed_json_skipped` | Bad JSON doesn't crash |
| UT-AI-12 | `test_sse_chunked_partial_lines` | Multi-byte boundary handling |
| UT-AI-12b | `test_sse_no_done_marker` | Stream without [DONE] still works |
| - | `test_sse_delta_no_content` | Delta with null/missing content |

### 4.8 URL Fetcher — 10 tests

| ID | Test | Validates |
|----|------|-----------|
| - | `test_extract_single_url` | Basic URL extraction |
| - | `test_extract_multiple_urls` | Multiple URLs from text |
| - | `test_extract_urls_deduplication` | Duplicate URLs removed |
| - | `test_extract_no_urls` | No URLs in plain text |
| - | `test_extract_urls_trailing_punctuation` | Strip trailing `.` `,` `)` |
| - | `test_extract_urls_with_query_params` | Query strings preserved |
| - | `test_extract_urls_preserves_balanced_parens` | Wikipedia-style URLs |
| - | `test_fetch_url_content_real` | Real HTTP fetch (example.com) |
| - | `test_fetch_all_urls_real` | Concurrent fetch integration |
| - | `test_fetch_url_content_invalid_url` | Invalid URL returns error |

### 4.9 Transfer (Import/Export) — 11 tests

| ID | Test | Validates |
|----|------|-----------|
| UT-TRANSFER-01 | `test_export_json_structure` | Format, version, topic, bots, messages; no api_key/base_url leak |
| UT-TRANSFER-02 | `test_export_base64_attachments` | Attachment encoded and decodable |
| UT-TRANSFER-03 | `test_export_skips_large_files` | >10MB → skipped=true with reason |
| UT-TRANSFER-04 | `test_export_handles_missing_file` | Missing file → skipped with "not found" |
| UT-TRANSFER-05 | `test_import_topic_suffix` | Title gets "(imported)" suffix |
| UT-TRANSFER-06 | `test_import_bot_matching` | Name match → linked; no match → sender_bot_id NULL |
| UT-TRANSFER-07 | `test_import_attachments` | Base64 decoded, written to disk |
| UT-TRANSFER-08 | `test_import_skips_skipped_attachments` | Skipped attachments not recreated |
| UT-TRANSFER-09 | `test_round_trip` | Export → import preserves all data |
| UT-TRANSFER-10 | `test_import_rejects_bad_format` | Wrong format string → error |
| UT-TRANSFER-11 | `test_import_rejects_bad_version` | Unsupported version → error |

## 5. Frontend Test Matrix (56 tests)

### 5.1 Smoke Test — 1 test

| File | Test | Validates |
|------|------|-----------|
| `smoke.test.ts` | Basic arithmetic | Test infrastructure works |

### 5.2 Zustand Store — 12 tests

| ID | Test | Validates |
|----|------|-----------|
| UT-STORE-01 | setBots replaces array | Bot state replacement |
| UT-STORE-02 | addBot appends | Bot addition |
| UT-STORE-03 | removeBot by id | Bot removal |
| UT-STORE-04 | updateBotInStore merges | Partial bot update |
| UT-STORE-05 | setTopics replaces | Topic state replacement |
| UT-STORE-06 | setActiveTopicId | Topic selection |
| UT-STORE-07 | setMessages replaces | Message state replacement |
| UT-STORE-08 | addMessage appends | Message addition |
| UT-STORE-09 | handleStreamEvent accumulates deltas | Streaming content build-up |
| UT-STORE-10 | handleStreamEvent sets done | Stream completion |
| UT-STORE-11 | clearStreaming resets | Streaming state cleanup |
| UT-STORE-12 | isAnyBotStreaming | Streaming status check |

### 5.3 Component Tests — 35 tests

**Sidebar (6 tests)**

| ID | Test | Validates |
|----|------|-----------|
| UT-COMP-01 | Renders topic list in order | Topic display |
| UT-COMP-01 | Shows "No topics yet" when empty | Empty state |
| UT-COMP-02 | Highlights active topic | Active selection |
| - | Clicking topic calls setActiveTopicId | Navigation |
| - | New Topic / Manage Bots buttons | Action buttons |
| - | Bot count and preview display | Topic metadata |

**MessageBubble (5 tests)**

| ID | Test | Validates |
|----|------|-----------|
| UT-COMP-03 | Human message: "You" label, right-aligned | Human styling |
| UT-COMP-04 | Bot message: bot name, left-aligned, avatar | Bot styling |
| UT-COMP-05 | Renders markdown content | Markdown support |
| UT-COMP-06 | Shows attachment indicators | Attachment display |
| - | Unknown sender_bot_id shows "Bot" | Fallback name |

**StreamingMessage (5 tests)**

| ID | Test | Validates |
|----|------|-----------|
| UT-COMP-07 | Shows thinking indicator | Pre-content state |
| UT-COMP-08 | Renders streaming content | Live content |
| UT-COMP-09 | Shows error state | Error display |
| - | Shows bot name with avatar | Bot identification |
| - | Shows retry info | Rate limit feedback |

**MessageInput (8 tests)**

| ID | Test | Validates |
|----|------|-----------|
| UT-COMP-10 | Renders textarea | Base rendering |
| UT-COMP-11 | Sends on Enter key | Keyboard send |
| UT-COMP-12 | Shift+Enter adds newline | Multi-line |
| UT-COMP-13 | Disabled during streaming | Loading state |
| UT-COMP-14 | Clears input after send | Input reset |
| UT-COMP-15 | @mention triggers autocomplete | Mention dropdown |
| - | Empty message not sent | Validation |
| - | Sends with content | Basic send |

**BotCard (4 tests)**

| ID | Test | Validates |
|----|------|-----------|
| UT-COMP-16 | Renders bot info | Name, model, avatar |
| - | Shows Vision badge | Vision flag display |
| - | Edit button calls onEdit | Edit action |
| - | Delete button calls onDelete | Delete action |

**BotFormDialog (2 tests)**

| ID | Test | Validates |
|----|------|-----------|
| UT-COMP-17 | Create mode: empty fields | New bot form |
| UT-COMP-18 | Edit mode: pre-filled fields | Edit bot form |

**CreateTopicDialog (5 tests)**

| ID | Test | Validates |
|----|------|-----------|
| UT-COMP-19 | Renders title input and bot checkboxes | Form rendering |
| UT-COMP-20 | Submit disabled without title | Validation |
| - | Submit disabled without bots | Validation |
| - | Submits with title and bots | Successful creation |
| - | Resets on close | Form cleanup |

### 5.4 Integration Tests — 8 tests

| ID | Test | Validates |
|----|------|-----------|
| IT-UI-01 | App renders sidebar and empty state | Initial layout |
| IT-UI-02 | Sidebar topic click navigates | Topic selection flow |
| IT-UI-03 | New Topic button opens dialog | Create topic flow |
| IT-UI-04 | Bot Manager opens from sidebar | Bot management flow |
| IT-UI-05 | Create Topic dialog validation | Form validation |
| IT-UI-06 | Streaming events update UI | Real-time streaming |
| IT-UI-07 | Multiple streams display correctly | Multi-bot streaming |
| IT-UI-08 | Stream completion clears states | Post-stream cleanup |

## 6. Test Coverage by Feature

| Feature | Rust Tests | Frontend Tests | Coverage |
|---------|-----------|---------------|----------|
| Bot CRUD | 10 | 6 | Full |
| Topic CRUD | 13 | 5 | Full |
| Messages | 6 | 5 | Full |
| Attachments | 6 | 1 | Backend: full, Frontend: basic |
| Chat/AI orchestration | 12 | 2 | Backend: full, Frontend: streaming only |
| SSE parsing | 8 | - | Backend: full |
| URL fetching | 10 | - | Backend: full |
| Import/Export | 11 | - | Backend: full, Frontend: not tested |
| Store state | - | 12 | Full |
| UI components | - | 35 | Core interactions covered |
| App integration | - | 8 | Key flows covered |

## 7. E2E Tests (Deferred)

WebdriverIO is configured (`wdio.conf.mjs`) but E2E tests require a running Tauri dev server. Planned test scenarios:

| ID | Scenario |
|----|----------|
| E2E-01 | Create bot → verify in list |
| E2E-02 | Create topic with bots → verify in sidebar |
| E2E-03 | Send message → verify in chat |
| E2E-04 | Bot streaming response → verify display |
| E2E-05 | Delete topic → verify removed |
| E2E-06 | @mention → verify only mentioned bot responds |
| E2E-07 | Export topic → verify file created |
| E2E-08 | Import topic → verify new topic appears |

## 8. Adding New Tests

### Rust

Add tests inside the `#[cfg(test)] mod tests` block in the relevant command file. Use the existing `setup_test_db()` pattern:

```rust
#[test]
fn test_my_feature() {
    let conn = setup_test_db();
    // ... test logic ...
}
```

### Frontend

Create test files at `src/components/<category>/__tests__/<Component>.test.tsx`. Use fixtures:

```typescript
import { makeBotFixture, makeMessageFixture } from "@/test/fixtures";

it("does something", () => {
    const bot = makeBotFixture({ name: "TestBot" });
    // ... render + assert ...
});
```
