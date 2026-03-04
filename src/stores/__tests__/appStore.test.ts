import { describe, it, expect, beforeEach } from "vitest";
import { useAppStore } from "../appStore";
import type { Bot, Message, StreamEvent } from "../../lib/tauri";

// ---------------------------------------------------------------------------
// Fixture helpers
// ---------------------------------------------------------------------------

function makeBotFixture(overrides: Partial<Bot> = {}): Bot {
  return {
    id: "bot-1",
    name: "Test Bot",
    avatar_color: "#ff0000",
    base_url: "https://api.example.com",
    api_key: "sk-test",
    model: "gpt-4",
    system_prompt: "You are a helpful assistant.",
    supports_vision: false,
    created_at: "2025-01-01T00:00:00Z",
    ...overrides,
  };
}

function makeMessageFixture(overrides: Partial<Message> = {}): Message {
  return {
    id: "msg-1",
    topic_id: "topic-1",
    sender_type: "human",
    sender_bot_id: null,
    content: "Hello",
    created_at: "2025-01-01T00:00:00Z",
    attachments: [],
    ...overrides,
  };
}

function makeStreamEventFixture(
  overrides: Partial<StreamEvent> = {},
): StreamEvent {
  return {
    topic_id: "topic-1",
    bot_id: "bot-1",
    bot_name: "Test Bot",
    delta: "",
    done: false,
    error: null,
    message_id: null,
    status: null,
    ...overrides,
  };
}

// ---------------------------------------------------------------------------
// Store initial state snapshot (used to reset between tests)
// ---------------------------------------------------------------------------

const initialState = useAppStore.getState();

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe("appStore", () => {
  beforeEach(() => {
    useAppStore.setState(initialState, true);
  });

  // ---- Bots ----------------------------------------------------------------

  describe("Bots", () => {
    it("UT-STORE-01: setBots replaces bot list", () => {
      const bots = [makeBotFixture({ id: "a" }), makeBotFixture({ id: "b" })];
      useAppStore.getState().setBots(bots);

      expect(useAppStore.getState().bots).toEqual(bots);

      // Replacing with a different list should fully overwrite
      const newBots = [makeBotFixture({ id: "c" })];
      useAppStore.getState().setBots(newBots);

      expect(useAppStore.getState().bots).toEqual(newBots);
      expect(useAppStore.getState().bots).toHaveLength(1);
    });

    it("UT-STORE-02: addBot appends to list", () => {
      const first = makeBotFixture({ id: "first" });
      const second = makeBotFixture({ id: "second" });

      useAppStore.getState().addBot(first);
      useAppStore.getState().addBot(second);

      const { bots } = useAppStore.getState();
      expect(bots).toHaveLength(2);
      expect(bots[0].id).toBe("first");
      expect(bots[1].id).toBe("second");
    });

    it("UT-STORE-03: removeBot filters by id", () => {
      const bots = [
        makeBotFixture({ id: "keep" }),
        makeBotFixture({ id: "remove" }),
        makeBotFixture({ id: "also-keep" }),
      ];
      useAppStore.getState().setBots(bots);
      useAppStore.getState().removeBot("remove");

      const remaining = useAppStore.getState().bots;
      expect(remaining).toHaveLength(2);
      expect(remaining.map((b) => b.id)).toEqual(["keep", "also-keep"]);
    });

    it("UT-STORE-04: updateBotInStore replaces matching bot", () => {
      const original = makeBotFixture({ id: "target", name: "Original" });
      const other = makeBotFixture({ id: "other", name: "Other" });
      useAppStore.getState().setBots([original, other]);

      const updated = makeBotFixture({ id: "target", name: "Updated" });
      useAppStore.getState().updateBotInStore(updated);

      const { bots } = useAppStore.getState();
      expect(bots).toHaveLength(2);
      // The matching bot should be updated in place
      expect(bots[0]).toEqual(updated);
      expect(bots[0].name).toBe("Updated");
      // The other bot should remain unchanged
      expect(bots[1]).toEqual(other);
    });
  });

  // ---- Topics ---------------------------------------------------------------

  describe("Topics", () => {
    it("UT-STORE-05: setActiveTopicId updates state", () => {
      expect(useAppStore.getState().activeTopicId).toBeNull();

      useAppStore.getState().setActiveTopicId("topic-42");
      expect(useAppStore.getState().activeTopicId).toBe("topic-42");

      // Setting to null should also work
      useAppStore.getState().setActiveTopicId(null);
      expect(useAppStore.getState().activeTopicId).toBeNull();
    });
  });

  // ---- Streaming ------------------------------------------------------------

  describe("Streaming", () => {
    it("UT-STORE-06: handleStreamEvent accumulates delta content", () => {
      // Must set the active topic so the event is not ignored
      useAppStore.getState().setActiveTopicId("topic-1");

      useAppStore.getState().handleStreamEvent(
        makeStreamEventFixture({ delta: "Hello" }),
      );
      useAppStore.getState().handleStreamEvent(
        makeStreamEventFixture({ delta: " world" }),
      );

      const state = useAppStore.getState().streamingStates["bot-1"];
      expect(state).toBeDefined();
      expect(state.content).toBe("Hello world");
    });

    it("UT-STORE-07: handleStreamEvent ignores wrong topic_id", () => {
      useAppStore.getState().setActiveTopicId("topic-1");

      useAppStore.getState().handleStreamEvent(
        makeStreamEventFixture({
          topic_id: "wrong-topic",
          delta: "should be ignored",
        }),
      );

      expect(useAppStore.getState().streamingStates).toEqual({});
    });

    it("UT-STORE-08: handleStreamEvent sets done=true on completion", () => {
      useAppStore.getState().setActiveTopicId("topic-1");

      // First send some content
      useAppStore.getState().handleStreamEvent(
        makeStreamEventFixture({ delta: "response" }),
      );

      // Then the completion event
      useAppStore.getState().handleStreamEvent(
        makeStreamEventFixture({ delta: "", done: true }),
      );

      const state = useAppStore.getState().streamingStates["bot-1"];
      expect(state.done).toBe(true);
      expect(state.content).toBe("response");
    });

    it("UT-STORE-09: handleStreamEvent stores error", () => {
      useAppStore.getState().setActiveTopicId("topic-1");

      useAppStore.getState().handleStreamEvent(
        makeStreamEventFixture({
          delta: "",
          done: true,
          error: "Rate limit exceeded",
        }),
      );

      const state = useAppStore.getState().streamingStates["bot-1"];
      expect(state.error).toBe("Rate limit exceeded");
      expect(state.done).toBe(true);
    });

    it("UT-STORE-10: isAnyBotStreaming returns true during streaming", () => {
      useAppStore.getState().setActiveTopicId("topic-1");

      // One bot still streaming (done=false)
      useAppStore.getState().handleStreamEvent(
        makeStreamEventFixture({
          bot_id: "bot-1",
          bot_name: "Bot 1",
          delta: "hi",
          done: false,
        }),
      );
      // Another bot finished
      useAppStore.getState().handleStreamEvent(
        makeStreamEventFixture({
          bot_id: "bot-2",
          bot_name: "Bot 2",
          delta: "done",
          done: true,
        }),
      );

      expect(useAppStore.getState().isAnyBotStreaming()).toBe(true);
    });

    it("UT-STORE-11: isAnyBotStreaming returns false when all done", () => {
      useAppStore.getState().setActiveTopicId("topic-1");

      useAppStore.getState().handleStreamEvent(
        makeStreamEventFixture({
          bot_id: "bot-1",
          bot_name: "Bot 1",
          delta: "a",
          done: true,
        }),
      );
      useAppStore.getState().handleStreamEvent(
        makeStreamEventFixture({
          bot_id: "bot-2",
          bot_name: "Bot 2",
          delta: "b",
          done: true,
        }),
      );

      expect(useAppStore.getState().isAnyBotStreaming()).toBe(false);
    });

    it("UT-STORE-12: clearStreaming resets all streaming states", () => {
      useAppStore.getState().setActiveTopicId("topic-1");

      // Add some streaming state
      useAppStore.getState().handleStreamEvent(
        makeStreamEventFixture({ delta: "data" }),
      );
      expect(
        Object.keys(useAppStore.getState().streamingStates).length,
      ).toBeGreaterThan(0);

      useAppStore.getState().clearStreaming();

      expect(useAppStore.getState().streamingStates).toEqual({});
    });
  });
});
