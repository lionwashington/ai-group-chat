import type { Bot, Message, TopicSummary, Attachment } from "@/lib/tauri";
import type { StreamingState } from "@/stores/appStore";

export function makeBotFixture(overrides: Partial<Bot> = {}): Bot {
  return {
    id: "bot-1",
    name: "TestBot",
    avatar_color: "#6366f1",
    base_url: "https://api.example.com/v1",
    api_key: "sk-test",
    model: "gpt-4o",
    system_prompt: "",
    supports_vision: false,
    created_at: "2026-01-01T00:00:00Z",
    ...overrides,
  };
}

export function makeMessageFixture(overrides: Partial<Message> = {}): Message {
  return {
    id: "msg-1",
    topic_id: "topic-1",
    sender_type: "human",
    sender_bot_id: null,
    content: "Hello world",
    created_at: "2026-01-01T00:00:00Z",
    attachments: [],
    ...overrides,
  };
}

export function makeTopicSummaryFixture(
  overrides: Partial<TopicSummary> = {},
): TopicSummary {
  return {
    id: "topic-1",
    title: "Test Topic",
    updated_at: "2026-01-01T00:00:00Z",
    bot_count: 2,
    last_message_preview: null,
    ...overrides,
  };
}

export function makeStreamingStateFixture(
  overrides: Partial<StreamingState> = {},
): StreamingState {
  return {
    botId: "bot-1",
    botName: "TestBot",
    content: "",
    done: false,
    error: null,
    status: null,
    retryInfo: null,
    ...overrides,
  };
}

export function makeAttachmentFixture(
  overrides: Partial<Attachment> = {},
): Attachment {
  return {
    id: "att-1",
    message_id: "msg-1",
    file_name: "document.pdf",
    file_path: "/tmp/document.pdf",
    file_type: "file",
    mime_type: "application/pdf",
    created_at: "2026-01-01T00:00:00Z",
    ...overrides,
  };
}
