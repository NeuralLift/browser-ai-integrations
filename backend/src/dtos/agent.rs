use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentRequest {
    #[serde(alias = "message")]
    pub query: String,
    pub session_id: Option<String>,
    #[serde(default)]
    pub stream: bool,
    pub image: Option<String>,
    pub custom_instruction: Option<String>,
    pub interactive_elements: Option<Vec<InteractiveElementDto>>,
    pub page_content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InteractiveElementDto {
    pub id: u32,
    pub role: String,
    pub name: String,
}
