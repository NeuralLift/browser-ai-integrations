use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum WsMessage {
    Ping,
    Pong,
    SessionUpdate {
        url: String,
        title: Option<String>,
    },
    #[serde(other)]
    Unknown,
}
