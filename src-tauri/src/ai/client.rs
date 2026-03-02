use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Clone)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub stream: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: ChatContent,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum ChatContent {
    Text(String),
    Parts(Vec<ContentPart>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum ContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrl },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageUrl {
    pub url: String,
}

pub async fn send_chat_request(
    base_url: &str,
    api_key: &str,
    request: &ChatRequest,
) -> Result<reqwest::Response, String> {
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
    let client = reqwest::Client::new();
    let mut builder = client.post(&url).header("Content-Type", "application/json");
    if !api_key.is_empty() {
        builder = builder.header("Authorization", format!("Bearer {}", api_key));
    }
    builder
        .json(request)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// UT-AI-13: ChatRequest serializes to valid JSON
    #[test]
    fn test_chat_request_serializes_to_valid_json() {
        let request = ChatRequest {
            model: "gpt-4".to_string(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: ChatContent::Text("You are helpful.".to_string()),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: ChatContent::Text("Hello!".to_string()),
                },
            ],
            stream: true,
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["model"], "gpt-4");
        assert_eq!(json["stream"], true);
        assert_eq!(json["messages"].as_array().unwrap().len(), 2);
        assert_eq!(json["messages"][0]["role"], "system");
        assert_eq!(json["messages"][0]["content"], "You are helpful.");
        assert_eq!(json["messages"][1]["role"], "user");
        assert_eq!(json["messages"][1]["content"], "Hello!");
    }

    /// UT-AI-13b: ChatRequest with image content serializes correctly
    #[test]
    fn test_chat_request_with_image_serializes() {
        let request = ChatRequest {
            model: "gpt-4-vision".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: ChatContent::Parts(vec![
                    ContentPart::Text {
                        text: "What is this?".to_string(),
                    },
                    ContentPart::ImageUrl {
                        image_url: ImageUrl {
                            url: "data:image/png;base64,abc123".to_string(),
                        },
                    },
                ]),
            }],
            stream: true,
        };

        let json = serde_json::to_value(&request).unwrap();
        let parts = json["messages"][0]["content"].as_array().unwrap();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0]["type"], "text");
        assert_eq!(parts[0]["text"], "What is this?");
        assert_eq!(parts[1]["type"], "image_url");
        assert_eq!(parts[1]["image_url"]["url"], "data:image/png;base64,abc123");
    }
}
