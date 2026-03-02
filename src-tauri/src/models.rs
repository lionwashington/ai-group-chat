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
