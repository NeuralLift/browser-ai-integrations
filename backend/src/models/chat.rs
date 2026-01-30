use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
}

#[derive(Deserialize)]
pub struct ChatRequest {
    pub message: String,
    pub custom_instruction: Option<String>,
    #[allow(dead_code)]
    pub image: Option<String>,
}

#[derive(Serialize)]
pub struct ChatResponse {
    pub response: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_tokens: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_tokens: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_tokens: Option<i32>,
}
