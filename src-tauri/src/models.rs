use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Bot {
    pub id: String,
    pub name: String,
    pub avatar_color: String,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub system_prompt: String,
    pub supports_vision: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateBotRequest {
    pub name: String,
    pub avatar_color: Option<String>,
    pub base_url: String,
    pub api_key: Option<String>,
    pub model: String,
    pub system_prompt: Option<String>,
    pub supports_vision: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBotRequest {
    pub name: Option<String>,
    pub avatar_color: Option<String>,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub system_prompt: Option<String>,
    pub supports_vision: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Topic {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub bots: Vec<Bot>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTopicRequest {
    pub title: String,
    pub bot_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TopicSummary {
    pub id: String,
    pub title: String,
    pub updated_at: String,
    pub bot_count: usize,
    pub last_message_preview: Option<String>,
}
