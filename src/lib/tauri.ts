import { invoke } from "@tauri-apps/api/core";

// ---------------------------------------------------------------------------
// Types matching Rust models
// ---------------------------------------------------------------------------

export interface Bot {
  id: string;
  name: string;
  avatar_color: string;
  base_url: string;
  api_key: string;
  model: string;
  system_prompt: string;
  supports_vision: boolean;
  created_at: string;
}

export interface TopicSummary {
  id: string;
  title: string;
  updated_at: string;
  bot_count: number;
  last_message_preview: string | null;
}

export interface Topic {
  id: string;
  title: string;
  created_at: string;
  updated_at: string;
  bots: Bot[];
}

export interface Attachment {
  id: string;
  message_id: string;
  file_name: string;
  file_path: string;
  file_type: string; // "image" | "file"
  mime_type: string;
  created_at: string;
}

export interface Message {
  id: string;
  topic_id: string;
  sender_type: string; // "human" | "bot"
  sender_bot_id: string | null;
  content: string;
  created_at: string;
  attachments: Attachment[];
}

export interface StreamEvent {
  topic_id: string;
  bot_id: string;
  bot_name: string;
  delta: string;
  done: boolean;
  error: string | null;
  message_id: string | null;
  status: string | null;
}

// ---------------------------------------------------------------------------
// Bot commands
// ---------------------------------------------------------------------------

export const listBots = () => invoke<Bot[]>("list_bots");

export const createBot = (req: {
  name: string;
  base_url: string;
  model: string;
  avatar_color?: string;
  api_key?: string;
  system_prompt?: string;
  supports_vision?: boolean;
}) => invoke<Bot>("create_bot", { req });

export const updateBot = (id: string, req: Record<string, unknown>) =>
  invoke<Bot>("update_bot", { id, req });

export const deleteBot = (id: string) => invoke<void>("delete_bot", { id });

// ---------------------------------------------------------------------------
// Topic commands
// ---------------------------------------------------------------------------

export const listTopics = () => invoke<TopicSummary[]>("list_topics");

export const getTopic = (id: string) => invoke<Topic>("get_topic", { id });

export const createTopic = (req: { title: string; bot_ids: string[] }) =>
  invoke<Topic>("create_topic", { req });

export const updateTopicBots = (topicId: string, botIds: string[]) =>
  invoke<Topic>("update_topic_bots", { topicId, botIds });

export const renameTopic = (id: string, title: string) =>
  invoke<Topic>("rename_topic", { id, title });

export const deleteTopic = (id: string) => invoke<void>("delete_topic", { id });

// ---------------------------------------------------------------------------
// Message commands
// ---------------------------------------------------------------------------

export const listMessages = (topicId: string) =>
  invoke<Message[]>("list_messages", { topicId });

export const sendHumanMessage = (req: { topic_id: string; content: string }) =>
  invoke<Message>("send_human_message", { req });

// ---------------------------------------------------------------------------
// Attachment commands
// ---------------------------------------------------------------------------

export const saveAttachment = (
  messageId: string,
  fileName: string,
  fileData: number[],
  mimeType: string,
) => invoke<Attachment>("save_attachment", { messageId, fileName, fileData, mimeType });

// ---------------------------------------------------------------------------
// Chat command
// ---------------------------------------------------------------------------

export const chatWithBots = (req: { topic_id: string; bot_ids?: string[] }) =>
  invoke<void>("chat_with_bots", { req });

// ---------------------------------------------------------------------------
// Import/Export commands
// ---------------------------------------------------------------------------

export const exportTopic = (topicId: string, filePath: string) =>
  invoke<void>("export_topic", { topicId, filePath });

export const importTopic = (filePath: string) =>
  invoke<string>("import_topic", { filePath });
