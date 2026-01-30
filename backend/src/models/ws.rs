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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_message_serialization() {
        let msg = WsMessage::Ping;
        let serialized = serde_json::to_string(&msg).unwrap();
        assert_eq!(serialized, r#"{"type":"Ping"}"#);
    }
}
