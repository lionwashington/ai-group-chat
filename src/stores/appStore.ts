import { create } from "zustand";
import type { Bot, TopicSummary, Topic, Message, StreamEvent } from "../lib/tauri";

export interface StreamingState {
  botId: string;
  botName: string;
  content: string;
  done: boolean;
  error: string | null;
  status: string | null;
  retryInfo: string | null;
}

interface AppState {
  // Bots
  bots: Bot[];
  setBots: (bots: Bot[]) => void;
  addBot: (bot: Bot) => void;
  removeBot: (id: string) => void;
  updateBotInStore: (bot: Bot) => void;

  // Topics
  topics: TopicSummary[];
  setTopics: (topics: TopicSummary[]) => void;
  activeTopicId: string | null;
  setActiveTopicId: (id: string | null) => void;
  activeTopic: Topic | null;
  setActiveTopic: (topic: Topic | null) => void;

  // Messages
  messages: Message[];
  setMessages: (messages: Message[]) => void;
  addMessage: (message: Message) => void;

  // Streaming
  streamingStates: Record<string, StreamingState>;
  handleStreamEvent: (event: StreamEvent) => void;
  clearStreaming: () => void;
  isAnyBotStreaming: () => boolean;
}

export const useAppStore = create<AppState>((set, get) => ({
  // -- Bots --
  bots: [],
  setBots: (bots) => set({ bots }),
  addBot: (bot) => set((state) => ({ bots: [...state.bots, bot] })),
  removeBot: (id) =>
    set((state) => ({ bots: state.bots.filter((b) => b.id !== id) })),
  updateBotInStore: (bot) =>
    set((state) => ({
      bots: state.bots.map((b) => (b.id === bot.id ? bot : b)),
    })),

  // -- Topics --
  topics: [],
  setTopics: (topics) => set({ topics }),
  activeTopicId: null,
  setActiveTopicId: (id) => set({ activeTopicId: id }),
  activeTopic: null,
  setActiveTopic: (topic) => set({ activeTopic: topic }),

  // -- Messages --
  messages: [],
  setMessages: (messages) => set({ messages }),
  addMessage: (message) =>
    set((state) => ({ messages: [...state.messages, message] })),

  // -- Streaming --
  streamingStates: {},

  handleStreamEvent: (event: StreamEvent) => {
    const { activeTopicId } = get();
    // Only process events for the currently active topic
    if (event.topic_id !== activeTopicId) return;

    set((state) => {
      const existing = state.streamingStates[event.bot_id];
      if (event.status === "retrying") {
        // Store retry info separately, don't append to content
        return {
          streamingStates: {
            ...state.streamingStates,
            [event.bot_id]: {
              botId: event.bot_id,
              botName: event.bot_name,
              content: existing?.content ?? "",
              done: event.done,
              error: event.error,
              status: event.status,
              retryInfo: event.delta,
            },
          },
        };
      }
      // Normal: append delta to content, clear retryInfo
      return {
        streamingStates: {
          ...state.streamingStates,
          [event.bot_id]: {
            botId: event.bot_id,
            botName: event.bot_name,
            content: (existing?.content ?? "") + event.delta,
            done: event.done,
            error: event.error,
            status: event.status,
            retryInfo: null,
          },
        },
      };
    });
  },

  clearStreaming: () => set({ streamingStates: {} }),

  isAnyBotStreaming: () => {
    const { streamingStates } = get();
    const states = Object.values(streamingStates);
    if (states.length === 0) return false;
    return states.some((s) => !s.done);
  },
}));
