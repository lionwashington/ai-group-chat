use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::ai::client::{
    send_chat_request, ChatContent, ChatMessage, ChatRequest, ContentPart, ImageUrl,
};
use crate::ai::stream::process_stream;
use crate::commands::message::{db_list_messages, db_save_bot_message};
use crate::db::DbState;
use crate::models::{Bot, Message};

// ---------------------------------------------------------------------------
// Event payload emitted to the frontend during streaming
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Clone)]
pub struct StreamEvent {
    pub topic_id: String,
    pub bot_id: String,
    pub bot_name: String,
    pub delta: String,
    pub done: bool,
    pub error: Option<String>,
    pub message_id: Option<String>,
}

// ---------------------------------------------------------------------------
// Request from the frontend
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ChatWithBotsRequest {
    pub topic_id: String,
    pub bot_ids: Option<Vec<String>>,
}

// ---------------------------------------------------------------------------
// build_chat_messages — convert DB history into OpenAI-compatible messages
// ---------------------------------------------------------------------------

/// Build the OpenAI-compatible messages array for a given bot from the
/// conversation history.
///
/// - Adds a system message from bot.system_prompt if non-empty
/// - Human messages become role "user", bot messages become role "assistant"
/// - For messages with image attachments:
///   - Vision-capable bot: ChatContent::Parts with ImageUrl (base64 data URL)
///   - Non-vision bot: text fallback "[Image attached: filename]"
/// - For messages with file attachments: reads file content, injects as text
pub fn build_chat_messages(messages: &[Message], bot: &Bot) -> Vec<ChatMessage> {
    let mut chat_messages = Vec::new();

    // System prompt
    if !bot.system_prompt.is_empty() {
        chat_messages.push(ChatMessage {
            role: "system".to_string(),
            content: ChatContent::Text(bot.system_prompt.clone()),
        });
    }

    for msg in messages {
        let role = if msg.sender_type == "human" {
            "user"
        } else {
            "assistant"
        };

        // Collect image and file attachments
        let mut image_parts: Vec<ContentPart> = Vec::new();
        let mut file_texts: Vec<String> = Vec::new();

        for att in &msg.attachments {
            if att.file_type == "image" {
                if bot.supports_vision {
                    // Try to read the image and encode as base64 data URL
                    if let Ok(bytes) = std::fs::read(&att.file_path) {
                        use base64::Engine;
                        let b64 =
                            base64::engine::general_purpose::STANDARD.encode(&bytes);
                        let data_url = format!("data:{};base64,{}", att.mime_type, b64);
                        image_parts.push(ContentPart::ImageUrl {
                            image_url: ImageUrl { url: data_url },
                        });
                    } else {
                        // Can't read the file, use fallback
                        file_texts.push(format!("[Image attached: {}]", att.file_name));
                    }
                } else {
                    // Non-vision bot: text fallback
                    file_texts.push(format!("[Image attached: {}]", att.file_name));
                }
            } else {
                // File attachment: read content and inject as text
                if let Ok(content) = std::fs::read_to_string(&att.file_path) {
                    file_texts.push(format!(
                        "[File: {}]\n```\n{}\n```",
                        att.file_name, content
                    ));
                } else if let Ok(bytes) = std::fs::read(&att.file_path) {
                    // Binary file: just note it
                    file_texts.push(format!(
                        "[File attached: {} ({} bytes)]",
                        att.file_name,
                        bytes.len()
                    ));
                } else {
                    file_texts.push(format!("[File attached: {}]", att.file_name));
                }
            }
        }

        // Build the content
        let has_images = !image_parts.is_empty();
        let has_file_text = !file_texts.is_empty();

        if has_images {
            // Use Parts content with text + images
            let mut parts = Vec::new();

            // Add the message text first
            let mut text = msg.content.clone();
            if has_file_text {
                text.push_str("\n\n");
                text.push_str(&file_texts.join("\n\n"));
            }
            parts.push(ContentPart::Text { text });

            // Add image parts
            parts.extend(image_parts);

            chat_messages.push(ChatMessage {
                role: role.to_string(),
                content: ChatContent::Parts(parts),
            });
        } else if has_file_text {
            // Text content with file attachments appended
            let mut text = msg.content.clone();
            text.push_str("\n\n");
            text.push_str(&file_texts.join("\n\n"));

            chat_messages.push(ChatMessage {
                role: role.to_string(),
                content: ChatContent::Text(text),
            });
        } else {
            // Plain text
            chat_messages.push(ChatMessage {
                role: role.to_string(),
                content: ChatContent::Text(msg.content.clone()),
            });
        }
    }

    chat_messages
}

// ---------------------------------------------------------------------------
// chat_with_bots — the main Tauri command
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn chat_with_bots(
    app: AppHandle,
    db: State<'_, DbState>,
    req: ChatWithBotsRequest,
) -> Result<(), String> {
    // 1. Load message history for the topic
    let (messages, bots) = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        let messages = db_list_messages(&conn, &req.topic_id)?;

        // Get topic to find associated bots
        let topic = crate::commands::topic::db_get_topic(&conn, &req.topic_id)?;

        // Determine which bots to call
        let bots: Vec<Bot> = if let Some(ref bot_ids) = req.bot_ids {
            topic
                .bots
                .into_iter()
                .filter(|b| bot_ids.contains(&b.id))
                .collect()
        } else {
            topic.bots
        };

        (messages, bots)
    };
    // DB lock is released here

    if bots.is_empty() {
        return Err("No bots available for this topic".to_string());
    }

    // 2. Spawn parallel tasks for each bot
    let mut handles = Vec::new();

    for bot in bots {
        let app_handle = app.clone();
        let topic_id = req.topic_id.clone();
        let messages = messages.clone();

        let handle = tokio::spawn(async move {
            // Build messages for this bot
            let chat_messages = build_chat_messages(&messages, &bot);

            let chat_request = ChatRequest {
                model: bot.model.clone(),
                messages: chat_messages,
                stream: true,
            };

            // Send the request
            let response = match send_chat_request(&bot.base_url, &bot.api_key, &chat_request).await
            {
                Ok(resp) => {
                    if !resp.status().is_success() {
                        let status = resp.status();
                        let body = resp.text().await.unwrap_or_default();
                        let error_msg =
                            format!("API error ({}): {}", status, body);
                        let _ = app_handle.emit(
                            "chat-stream",
                            StreamEvent {
                                topic_id: topic_id.clone(),
                                bot_id: bot.id.clone(),
                                bot_name: bot.name.clone(),
                                delta: String::new(),
                                done: true,
                                error: Some(error_msg),
                                message_id: None,
                            },
                        );
                        return;
                    }
                    resp
                }
                Err(e) => {
                    let _ = app_handle.emit(
                        "chat-stream",
                        StreamEvent {
                            topic_id: topic_id.clone(),
                            bot_id: bot.id.clone(),
                            bot_name: bot.name.clone(),
                            delta: String::new(),
                            done: true,
                            error: Some(e),
                            message_id: None,
                        },
                    );
                    return;
                }
            };

            // Process the stream
            let bot_id_for_closure = bot.id.clone();
            let bot_name_for_closure = bot.name.clone();
            let topic_id_for_closure = topic_id.clone();
            let app_for_closure = app_handle.clone();

            let result = process_stream(response, |delta| {
                let _ = app_for_closure.emit(
                    "chat-stream",
                    StreamEvent {
                        topic_id: topic_id_for_closure.clone(),
                        bot_id: bot_id_for_closure.clone(),
                        bot_name: bot_name_for_closure.clone(),
                        delta: delta.to_string(),
                        done: false,
                        error: None,
                        message_id: None,
                    },
                );
            })
            .await;

            match result {
                Ok(full_content) => {
                    // Save to DB
                    let message_id = {
                        let db = app_handle.state::<DbState>();
                        let conn = db.0.lock().unwrap();
                        match db_save_bot_message(
                            &conn,
                            &topic_id,
                            &bot.id,
                            &full_content,
                        ) {
                            Ok(msg) => Some(msg.id),
                            Err(_) => None,
                        }
                    };

                    // Emit done event
                    let _ = app_handle.emit(
                        "chat-stream",
                        StreamEvent {
                            topic_id: topic_id.clone(),
                            bot_id: bot.id.clone(),
                            bot_name: bot.name.clone(),
                            delta: String::new(),
                            done: true,
                            error: None,
                            message_id,
                        },
                    );
                }
                Err(e) => {
                    let _ = app_handle.emit(
                        "chat-stream",
                        StreamEvent {
                            topic_id: topic_id.clone(),
                            bot_id: bot.id.clone(),
                            bot_name: bot.name.clone(),
                            delta: String::new(),
                            done: true,
                            error: Some(e),
                            message_id: None,
                        },
                    );
                }
            }
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete (non-blocking from Tauri's perspective
    // since this whole function is async)
    for handle in handles {
        let _ = handle.await;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Attachment, Bot, Message};

    fn make_test_bot(supports_vision: bool, system_prompt: &str) -> Bot {
        Bot {
            id: "bot-1".to_string(),
            name: "TestBot".to_string(),
            avatar_color: "#6366f1".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            api_key: "sk-test".to_string(),
            model: "gpt-4".to_string(),
            system_prompt: system_prompt.to_string(),
            supports_vision,
            created_at: "2024-01-01T00:00:00Z".to_string(),
        }
    }

    fn make_human_message(content: &str) -> Message {
        Message {
            id: "msg-h1".to_string(),
            topic_id: "topic-1".to_string(),
            sender_type: "human".to_string(),
            sender_bot_id: None,
            content: content.to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            attachments: Vec::new(),
        }
    }

    fn make_bot_message(content: &str, bot_id: &str) -> Message {
        Message {
            id: "msg-b1".to_string(),
            topic_id: "topic-1".to_string(),
            sender_type: "bot".to_string(),
            sender_bot_id: Some(bot_id.to_string()),
            content: content.to_string(),
            created_at: "2024-01-01T00:01:00Z".to_string(),
            attachments: Vec::new(),
        }
    }

    /// UT-AI-01: Build messages with system prompt -> system message first
    #[test]
    fn test_build_messages_with_system_prompt() {
        let bot = make_test_bot(false, "You are a helpful assistant.");
        let messages = vec![make_human_message("Hello!")];

        let result = build_chat_messages(&messages, &bot);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].role, "system");
        match &result[0].content {
            ChatContent::Text(t) => assert_eq!(t, "You are a helpful assistant."),
            _ => panic!("Expected text content for system message"),
        }
        assert_eq!(result[1].role, "user");
    }

    /// UT-AI-02: Build messages without system prompt -> no system message
    #[test]
    fn test_build_messages_without_system_prompt() {
        let bot = make_test_bot(false, "");
        let messages = vec![make_human_message("Hello!")];

        let result = build_chat_messages(&messages, &bot);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].role, "user");
    }

    /// UT-AI-03: Build messages from mixed human/bot history -> correct roles
    #[test]
    fn test_build_messages_mixed_history() {
        let bot = make_test_bot(false, "");
        let messages = vec![
            make_human_message("Hello!"),
            make_bot_message("Hi there!", "bot-1"),
            make_human_message("How are you?"),
            make_bot_message("I'm doing well!", "bot-2"),
        ];

        let result = build_chat_messages(&messages, &bot);

        assert_eq!(result.len(), 4);
        assert_eq!(result[0].role, "user");
        assert_eq!(result[1].role, "assistant");
        assert_eq!(result[2].role, "user");
        assert_eq!(result[3].role, "assistant");

        // Verify content
        match &result[0].content {
            ChatContent::Text(t) => assert_eq!(t, "Hello!"),
            _ => panic!("Expected text"),
        }
        match &result[1].content {
            ChatContent::Text(t) => assert_eq!(t, "Hi there!"),
            _ => panic!("Expected text"),
        }
    }

    /// UT-AI-04: Build messages with image (vision bot) -> image_url content part
    #[test]
    fn test_build_messages_image_vision_bot() {
        let bot = make_test_bot(true, "");

        // Create a temp file for the image
        let temp_dir =
            std::env::temp_dir().join(format!("test_ai_img_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let img_path = temp_dir.join("test.png");
        std::fs::write(&img_path, b"fake-png-data").unwrap();

        let mut msg = make_human_message("Look at this!");
        msg.attachments.push(Attachment {
            id: "att-1".to_string(),
            message_id: msg.id.clone(),
            file_name: "test.png".to_string(),
            file_path: img_path.to_string_lossy().to_string(),
            file_type: "image".to_string(),
            mime_type: "image/png".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
        });

        let messages = vec![msg];
        let result = build_chat_messages(&messages, &bot);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].role, "user");

        match &result[0].content {
            ChatContent::Parts(parts) => {
                assert_eq!(parts.len(), 2); // text + image
                match &parts[0] {
                    ContentPart::Text { text } => assert_eq!(text, "Look at this!"),
                    _ => panic!("Expected text part first"),
                }
                match &parts[1] {
                    ContentPart::ImageUrl { image_url } => {
                        assert!(image_url.url.starts_with("data:image/png;base64,"));
                    }
                    _ => panic!("Expected image_url part"),
                }
            }
            _ => panic!("Expected Parts content for vision bot with image"),
        }

        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    /// UT-AI-05: Build messages with image (non-vision bot) -> text fallback
    #[test]
    fn test_build_messages_image_non_vision_bot() {
        let bot = make_test_bot(false, "");

        // Create a temp file for the image
        let temp_dir =
            std::env::temp_dir().join(format!("test_ai_novis_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let img_path = temp_dir.join("photo.jpg");
        std::fs::write(&img_path, b"fake-jpg-data").unwrap();

        let mut msg = make_human_message("Check this photo");
        msg.attachments.push(Attachment {
            id: "att-2".to_string(),
            message_id: msg.id.clone(),
            file_name: "photo.jpg".to_string(),
            file_path: img_path.to_string_lossy().to_string(),
            file_type: "image".to_string(),
            mime_type: "image/jpeg".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
        });

        let messages = vec![msg];
        let result = build_chat_messages(&messages, &bot);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].role, "user");

        match &result[0].content {
            ChatContent::Text(t) => {
                assert!(t.contains("Check this photo"));
                assert!(t.contains("[Image attached: photo.jpg]"));
            }
            _ => panic!("Expected text content for non-vision bot"),
        }

        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    /// UT-AI-06: Build messages with file attachment -> text block injected
    #[test]
    fn test_build_messages_file_attachment() {
        let bot = make_test_bot(false, "");

        // Create a temp file
        let temp_dir =
            std::env::temp_dir().join(format!("test_ai_file_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let file_path = temp_dir.join("code.py");
        std::fs::write(&file_path, "print('hello world')").unwrap();

        let mut msg = make_human_message("Review this code");
        msg.attachments.push(Attachment {
            id: "att-3".to_string(),
            message_id: msg.id.clone(),
            file_name: "code.py".to_string(),
            file_path: file_path.to_string_lossy().to_string(),
            file_type: "file".to_string(),
            mime_type: "text/x-python".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
        });

        let messages = vec![msg];
        let result = build_chat_messages(&messages, &bot);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].role, "user");

        match &result[0].content {
            ChatContent::Text(t) => {
                assert!(t.contains("Review this code"));
                assert!(t.contains("[File: code.py]"));
                assert!(t.contains("print('hello world')"));
            }
            _ => panic!("Expected text content with file injection"),
        }

        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    /// UT-AI-06b: Build messages with both image and file attachments on vision bot
    #[test]
    fn test_build_messages_mixed_attachments_vision() {
        let bot = make_test_bot(true, "Helper");

        let temp_dir =
            std::env::temp_dir().join(format!("test_ai_mixed_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        let img_path = temp_dir.join("diagram.png");
        std::fs::write(&img_path, b"png-data").unwrap();

        let file_path = temp_dir.join("notes.txt");
        std::fs::write(&file_path, "Some notes here").unwrap();

        let mut msg = make_human_message("Explain this");
        msg.attachments.push(Attachment {
            id: "att-img".to_string(),
            message_id: msg.id.clone(),
            file_name: "diagram.png".to_string(),
            file_path: img_path.to_string_lossy().to_string(),
            file_type: "image".to_string(),
            mime_type: "image/png".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
        });
        msg.attachments.push(Attachment {
            id: "att-file".to_string(),
            message_id: msg.id.clone(),
            file_name: "notes.txt".to_string(),
            file_path: file_path.to_string_lossy().to_string(),
            file_type: "file".to_string(),
            mime_type: "text/plain".to_string(),
            created_at: "2024-01-01T00:00:01Z".to_string(),
        });

        let messages = vec![msg];
        let result = build_chat_messages(&messages, &bot);

        // Should have system + user message
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].role, "system");
        assert_eq!(result[1].role, "user");

        match &result[1].content {
            ChatContent::Parts(parts) => {
                // text part (message + file text) + image part
                assert!(parts.len() >= 2);
                match &parts[0] {
                    ContentPart::Text { text } => {
                        assert!(text.contains("Explain this"));
                        assert!(text.contains("[File: notes.txt]"));
                        assert!(text.contains("Some notes here"));
                    }
                    _ => panic!("Expected text part first"),
                }
                // Should have an image part
                let has_image = parts.iter().any(|p| matches!(p, ContentPart::ImageUrl { .. }));
                assert!(has_image, "Should have image_url part for vision bot");
            }
            _ => panic!("Expected Parts content for vision bot with images"),
        }

        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
