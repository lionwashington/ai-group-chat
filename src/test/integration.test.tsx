import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, waitFor, act, within, fireEvent } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import App from "@/App";
import { useAppStore } from "@/stores/appStore";
import {
  makeBotFixture,
  makeMessageFixture,
  makeTopicSummaryFixture,
  makeAttachmentFixture,
} from "@/test/fixtures";
import type { Bot, Topic, TopicSummary, Message, Attachment } from "@/lib/tauri";

// ---------------------------------------------------------------------------
// Mock react-markdown and plugins (they use ESM / node APIs not available in jsdom)
// ---------------------------------------------------------------------------
vi.mock("react-markdown", () => ({
  default: ({ children }: { children: string }) => <p>{children}</p>,
}));
vi.mock("remark-gfm", () => ({ default: () => {} }));
vi.mock("rehype-highlight", () => ({ default: () => {} }));

// ---------------------------------------------------------------------------
// Typed mock references
// ---------------------------------------------------------------------------
const mockInvoke = vi.mocked(invoke);
const mockListen = vi.mocked(listen);

// ---------------------------------------------------------------------------
// Test data
// ---------------------------------------------------------------------------
const bot1: Bot = makeBotFixture({ id: "bot-1", name: "GPT-4o", avatar_color: "#6366f1", model: "gpt-4o" });
const bot2: Bot = makeBotFixture({ id: "bot-2", name: "Claude", avatar_color: "#ec4899", model: "claude-3" });

const topicSummary1: TopicSummary = makeTopicSummaryFixture({
  id: "topic-1",
  title: "Test Topic",
  bot_count: 2,
  last_message_preview: "Hello!",
});

const topic1: Topic = {
  id: "topic-1",
  title: "Test Topic",
  created_at: "2026-01-01T00:00:00Z",
  updated_at: "2026-01-01T00:00:00Z",
  bots: [bot1, bot2],
};

const humanMessage: Message = makeMessageFixture({
  id: "msg-1",
  topic_id: "topic-1",
  sender_type: "human",
  sender_bot_id: null,
  content: "Hello bots!",
});

const botMessage: Message = makeMessageFixture({
  id: "msg-2",
  topic_id: "topic-1",
  sender_type: "bot",
  sender_bot_id: "bot-1",
  content: "Hello human!",
});

// ---------------------------------------------------------------------------
// Store initial state snapshot (captured once for resets)
// ---------------------------------------------------------------------------
const storeInitialState = useAppStore.getInitialState();

// ---------------------------------------------------------------------------
// Helper: capture the chat-stream listener callback
// ---------------------------------------------------------------------------
let streamCallback: ((event: { payload: unknown }) => void) | null = null;

function setupMocks(overrides: Record<string, unknown> = {}) {
  streamCallback = null;

  mockListen.mockImplementation(async (event: string, handler: any) => {
    if (event === "chat-stream") {
      streamCallback = handler;
    }
    return () => {};
  });

  mockInvoke.mockImplementation(async (cmd: string, args?: any) => {
    // Allow per-test overrides
    if (cmd in overrides) {
      const val = overrides[cmd];
      return typeof val === "function" ? val(args) : val;
    }
    switch (cmd) {
      case "list_bots":
        return [bot1, bot2];
      case "list_topics":
        return [topicSummary1];
      case "get_topic":
        return topic1;
      case "list_messages":
        return [humanMessage, botMessage];
      case "send_human_message":
        return humanMessage;
      case "chat_with_bots":
        return undefined;
      case "create_bot":
        return { ...bot1, ...args?.req, id: "bot-new" };
      case "create_topic":
        return { id: "topic-new", title: args?.req?.title ?? "New", created_at: "2026-01-01T00:00:00Z", updated_at: "2026-01-01T00:00:00Z", bots: [bot1] };
      case "save_attachment":
        return makeAttachmentFixture({ message_id: args?.messageId });
      default:
        return undefined;
    }
  });
}

// ---------------------------------------------------------------------------
// Reset between tests
// ---------------------------------------------------------------------------
beforeEach(() => {
  vi.clearAllMocks();
  // Reset Zustand store to pristine state (merge mode preserves methods)
  useAppStore.setState(storeInitialState);
  setupMocks();
});

// ---------------------------------------------------------------------------
// IT-UI-01: App loads -> fetches bots and topics
// ---------------------------------------------------------------------------
describe("IT-UI-01: App loads -> fetches bots and topics", () => {
  it("calls listBots and listTopics on mount", async () => {
    render(<App />);

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("list_bots");
      expect(mockInvoke).toHaveBeenCalledWith("list_topics");
    });

    // Bots are stored
    expect(useAppStore.getState().bots).toEqual([bot1, bot2]);
    // Topics are stored
    expect(useAppStore.getState().topics).toEqual([topicSummary1]);
  });

  it("renders topics in the sidebar after loading", async () => {
    render(<App />);

    await waitFor(() => {
      expect(screen.getByText("Test Topic")).toBeInTheDocument();
    });
  });
});

// ---------------------------------------------------------------------------
// IT-UI-02: Click topic -> loads messages
// ---------------------------------------------------------------------------
describe("IT-UI-02: Click topic -> loads messages", () => {
  it("calls getTopic and listMessages when a topic is clicked, and renders messages", async () => {
    const user = userEvent.setup();
    render(<App />);

    // Wait for topics to appear
    await waitFor(() => {
      expect(screen.getByText("Test Topic")).toBeInTheDocument();
    });

    // Click the topic
    await user.click(screen.getByText("Test Topic"));

    // Verify the right IPC calls were made
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("get_topic", { id: "topic-1" });
      expect(mockInvoke).toHaveBeenCalledWith("list_messages", { topicId: "topic-1" });
    });

    // Messages should be rendered
    await waitFor(() => {
      expect(screen.getByText("Hello bots!")).toBeInTheDocument();
      expect(screen.getByText("Hello human!")).toBeInTheDocument();
    });
  });
});

// ---------------------------------------------------------------------------
// IT-UI-03: Create bot via dialog -> appears in list
// ---------------------------------------------------------------------------
describe("IT-UI-03: Create bot via dialog -> appears in list", () => {
  it("opens BotManager, adds a new bot, and the bot appears in the store", async () => {
    const user = userEvent.setup({ pointerEventsCheck: 0 });

    const newBot: Bot = makeBotFixture({
      id: "bot-new",
      name: "NewBot",
      base_url: "https://api.test.com/v1",
      model: "test-model",
    });

    // After bot creation, list_bots should include the new bot
    let botCreated = false;
    setupMocks({
      create_bot: (args: any) => {
        botCreated = true;
        return newBot;
      },
      list_bots: () => {
        return botCreated ? [bot1, bot2, newBot] : [bot1, bot2];
      },
    });

    render(<App />);

    // Wait for initial load
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("list_bots");
    });

    // Click "Manage Bots" to open BotManager dialog
    await user.click(screen.getByRole("button", { name: /manage bots/i }));

    // The dialog should open showing "Manage Bots" title
    await waitFor(() => {
      expect(screen.getByRole("heading", { name: /manage bots/i })).toBeInTheDocument();
    });

    // Click "Add Bot" button in the dialog
    await user.click(screen.getByRole("button", { name: /add bot/i }));

    // Wait for the BotFormDialog to appear (it has input#bot-name)
    await waitFor(() => {
      expect(document.getElementById("bot-name")).toBeInTheDocument();
    });

    // Fill in the form - use userEvent with pointerEventsCheck disabled for nested dialog
    const nameInput = document.getElementById("bot-name")! as HTMLInputElement;
    const urlInput = document.getElementById("bot-url")! as HTMLInputElement;
    const modelInput = document.getElementById("bot-model")! as HTMLInputElement;

    await user.type(nameInput, "NewBot");
    await user.type(urlInput, "https://api.test.com/v1");
    await user.type(modelInput, "test-model");

    // Submit the form directly
    const form = nameInput.closest("form")!;
    expect(form).toBeTruthy();
    fireEvent.submit(form);

    // Verify createBot was called
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("create_bot", {
        req: expect.objectContaining({
          name: "NewBot",
          base_url: "https://api.test.com/v1",
          model: "test-model",
        }),
      });
    });

    // Verify the bot was added to store
    await waitFor(() => {
      const bots = useAppStore.getState().bots;
      expect(bots.some((b) => b.id === "bot-new")).toBe(true);
    });
  });
});

// ---------------------------------------------------------------------------
// IT-UI-04: Create topic via dialog -> appears in sidebar
// ---------------------------------------------------------------------------
describe("IT-UI-04: Create topic via dialog -> appears in sidebar", () => {
  it("opens CreateTopicDialog, fills in details, and the topic appears in sidebar", async () => {
    const user = userEvent.setup();

    // After topic creation, listTopics will return updated list
    const newTopicSummary: TopicSummary = makeTopicSummaryFixture({
      id: "topic-new",
      title: "My New Topic",
      bot_count: 1,
    });

    let listTopicsCallCount = 0;
    setupMocks({
      list_topics: () => {
        listTopicsCallCount++;
        // First call returns original, subsequent calls return with new topic
        return listTopicsCallCount <= 1
          ? [topicSummary1]
          : [topicSummary1, newTopicSummary];
      },
    });

    render(<App />);

    // Wait for initial load
    await waitFor(() => {
      expect(screen.getByText("Test Topic")).toBeInTheDocument();
    });

    // Click "New Topic" button
    await user.click(screen.getByRole("button", { name: /new topic/i }));

    // Dialog should open
    await waitFor(() => {
      expect(screen.getByText("New Topic", { selector: "[data-slot='dialog-title']" })).toBeInTheDocument();
    });

    // Fill in title
    await user.type(screen.getByLabelText(/title/i), "My New Topic");

    // Select a bot (click the checkbox for bot1)
    const botCheckboxes = screen.getAllByRole("checkbox");
    await user.click(botCheckboxes[0]);

    // Submit
    await user.click(screen.getByRole("button", { name: /create topic/i }));

    // Verify createTopic was called
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("create_topic", {
        req: expect.objectContaining({
          title: "My New Topic",
        }),
      });
    });

    // After creation, listTopics is called again and the new topic should appear
    await waitFor(() => {
      expect(screen.getByText("My New Topic")).toBeInTheDocument();
    });
  });
});

// ---------------------------------------------------------------------------
// IT-UI-05: Send message -> bot streams response
// ---------------------------------------------------------------------------
describe("IT-UI-05: Send message -> bot streams response", () => {
  it("sends a human message, then stream events update the UI", async () => {
    const user = userEvent.setup();
    render(<App />);

    // Wait for load and click topic
    await waitFor(() => {
      expect(screen.getByText("Test Topic")).toBeInTheDocument();
    });
    await user.click(screen.getByText("Test Topic"));

    // Wait for messages to load
    await waitFor(() => {
      expect(screen.getByText("Hello bots!")).toBeInTheDocument();
    });

    // Type and send a message (Enter sends)
    const textarea = screen.getByPlaceholderText(/type a message/i);
    await user.type(textarea, "What is AI?{Enter}");

    // Verify sendHumanMessage was called
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("send_human_message", {
        req: { topic_id: "topic-1", content: "What is AI?" },
      });
    });

    // Verify chatWithBots was called
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("chat_with_bots", {
        req: { topic_id: "topic-1" },
      });
    });

    // Simulate streaming events
    expect(streamCallback).not.toBeNull();

    await act(async () => {
      streamCallback!({
        payload: {
          topic_id: "topic-1",
          bot_id: "bot-1",
          bot_name: "GPT-4o",
          delta: "Hello ",
          done: false,
          error: null,
          message_id: null,
        },
      });
    });

    // Streaming content should appear
    await waitFor(() => {
      expect(screen.getByText("Hello")).toBeInTheDocument();
    });

    // More streaming
    await act(async () => {
      streamCallback!({
        payload: {
          topic_id: "topic-1",
          bot_id: "bot-1",
          bot_name: "GPT-4o",
          delta: "world!",
          done: false,
          error: null,
          message_id: null,
        },
      });
    });

    await waitFor(() => {
      expect(screen.getByText("Hello world!")).toBeInTheDocument();
    });
  });
});

// ---------------------------------------------------------------------------
// IT-UI-06: Multiple bots streaming -> all visible simultaneously
// ---------------------------------------------------------------------------
describe("IT-UI-06: Multiple bots streaming -> all visible simultaneously", () => {
  it("renders streaming messages from multiple bots at the same time", async () => {
    const user = userEvent.setup();
    render(<App />);

    // Load and select topic
    await waitFor(() => {
      expect(screen.getByText("Test Topic")).toBeInTheDocument();
    });
    await user.click(screen.getByText("Test Topic"));

    await waitFor(() => {
      expect(screen.getByText("Hello bots!")).toBeInTheDocument();
    });

    expect(streamCallback).not.toBeNull();

    // Simulate two bots streaming simultaneously
    await act(async () => {
      streamCallback!({
        payload: {
          topic_id: "topic-1",
          bot_id: "bot-1",
          bot_name: "GPT-4o",
          delta: "Response from GPT",
          done: false,
          error: null,
          message_id: null,
        },
      });
    });

    await act(async () => {
      streamCallback!({
        payload: {
          topic_id: "topic-1",
          bot_id: "bot-2",
          bot_name: "Claude",
          delta: "Response from Claude",
          done: false,
          error: null,
          message_id: null,
        },
      });
    });

    // Both streaming messages should be visible
    await waitFor(() => {
      expect(screen.getByText("Response from GPT")).toBeInTheDocument();
      expect(screen.getByText("Response from Claude")).toBeInTheDocument();
    });

    // Both bot names should be visible in streaming area
    // The StreamingMessage components show the bot name
    const gptNames = screen.getAllByText("GPT-4o");
    const claudeNames = screen.getAllByText("Claude");
    // At least one instance of each should be in a streaming message
    expect(gptNames.length).toBeGreaterThanOrEqual(1);
    expect(claudeNames.length).toBeGreaterThanOrEqual(1);
  });
});

// ---------------------------------------------------------------------------
// IT-UI-07: Stream completes -> messages reloaded from DB
// ---------------------------------------------------------------------------
describe("IT-UI-07: Stream completes -> messages reloaded from DB", () => {
  it("calls listMessages after all bots finish streaming", async () => {
    const user = userEvent.setup();

    // Track listMessages calls - we need to distinguish the reload call
    let listMessagesCallCount = 0;
    setupMocks({
      list_messages: () => {
        listMessagesCallCount++;
        return [humanMessage, botMessage];
      },
    });

    render(<App />);

    // Load and select topic
    await waitFor(() => {
      expect(screen.getByText("Test Topic")).toBeInTheDocument();
    });
    await user.click(screen.getByText("Test Topic"));

    await waitFor(() => {
      expect(screen.getByText("Hello bots!")).toBeInTheDocument();
    });

    expect(streamCallback).not.toBeNull();

    // Record the call count after initial load
    const countAfterLoad = listMessagesCallCount;

    // Start streaming from both bots
    await act(async () => {
      streamCallback!({
        payload: {
          topic_id: "topic-1",
          bot_id: "bot-1",
          bot_name: "GPT-4o",
          delta: "bot1 response",
          done: false,
          error: null,
          message_id: null,
        },
      });
    });

    await act(async () => {
      streamCallback!({
        payload: {
          topic_id: "topic-1",
          bot_id: "bot-2",
          bot_name: "Claude",
          delta: "bot2 response",
          done: false,
          error: null,
          message_id: null,
        },
      });
    });

    // Complete bot-1
    await act(async () => {
      streamCallback!({
        payload: {
          topic_id: "topic-1",
          bot_id: "bot-1",
          bot_name: "GPT-4o",
          delta: "",
          done: true,
          error: null,
          message_id: "msg-bot1",
        },
      });
    });

    // Complete bot-2
    await act(async () => {
      streamCallback!({
        payload: {
          topic_id: "topic-1",
          bot_id: "bot-2",
          bot_name: "Claude",
          delta: "",
          done: true,
          error: null,
          message_id: "msg-bot2",
        },
      });
    });

    // After all bots are done, ChatView should reload messages (with a 300ms delay)
    // Wait for the reload to happen
    await waitFor(
      () => {
        expect(listMessagesCallCount).toBeGreaterThan(countAfterLoad);
      },
      { timeout: 2000 },
    );
  });
});

// ---------------------------------------------------------------------------
// IT-UI-08: Upload file -> preview shown -> sent with message
// ---------------------------------------------------------------------------
describe("IT-UI-08: Upload file -> preview shown -> sent with message", () => {
  it("shows file preview and calls saveAttachment after sending", async () => {
    const user = userEvent.setup();
    render(<App />);

    // Wait for load and select topic
    await waitFor(() => {
      expect(screen.getByText("Test Topic")).toBeInTheDocument();
    });
    await user.click(screen.getByText("Test Topic"));

    await waitFor(() => {
      expect(screen.getByText("Hello bots!")).toBeInTheDocument();
    });

    // Find the hidden file input
    const fileInput = document.querySelector('input[type="file"]') as HTMLInputElement;
    expect(fileInput).toBeTruthy();

    // Create a test file
    const testFile = new File(["file content"], "test-doc.pdf", {
      type: "application/pdf",
    });

    // Upload the file
    await user.upload(fileInput, testFile);

    // File preview should appear
    await waitFor(() => {
      expect(screen.getByText("test-doc.pdf")).toBeInTheDocument();
    });

    // Type a message
    const textarea = screen.getByPlaceholderText(/type a message/i);
    await user.type(textarea, "Check this file{Enter}");

    // Verify sendHumanMessage was called
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("send_human_message", {
        req: { topic_id: "topic-1", content: "Check this file" },
      });
    });

    // Verify saveAttachment was called with the file info
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("save_attachment", {
        messageId: "msg-1",
        fileName: "test-doc.pdf",
        fileData: expect.any(Array),
        mimeType: "application/pdf",
      });
    });
  });
});
