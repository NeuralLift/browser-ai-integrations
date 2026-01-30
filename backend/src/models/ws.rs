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
    ActionCommand(ActionCommand),
    ActionResult(ActionResult),
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ActionCommand {
    #[serde(rename = "navigate_to")]
    NavigateTo { url: String },
    #[serde(rename = "click_element")]
    ClickElement {
        #[serde(rename = "ref")]
        ref_id: i32,
    },
    #[serde(rename = "type_text")]
    TypeText {
        #[serde(rename = "ref")]
        ref_id: i32,
        text: String,
    },
    #[serde(rename = "scroll_to")]
    ScrollTo { x: i32, y: i32 },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ActionResult {
    pub success: bool,
    pub error: Option<String>,
    pub data: Option<serde_json::Value>,
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

    #[test]
    fn test_action_command_serialization() {
        let cmd = WsMessage::ActionCommand(ActionCommand::ClickElement { ref_id: 1 });
        let serialized = serde_json::to_string(&cmd).unwrap();
        assert_eq!(
            serialized,
            r#"{"type":"ActionCommand","data":{"type":"click_element","ref":1}}"#
        );

        let cmd = WsMessage::ActionCommand(ActionCommand::NavigateTo {
            url: "https://example.com".to_string(),
        });
        let serialized = serde_json::to_string(&cmd).unwrap();
        assert_eq!(
            serialized,
            r#"{"type":"ActionCommand","data":{"type":"navigate_to","url":"https://example.com"}}"#
        );

        let cmd = WsMessage::ActionCommand(ActionCommand::TypeText {
            ref_id: 2,
            text: "Hello".to_string(),
        });
        let serialized = serde_json::to_string(&cmd).unwrap();
        assert_eq!(
            serialized,
            r#"{"type":"ActionCommand","data":{"type":"type_text","ref":2,"text":"Hello"}}"#
        );

        let cmd = WsMessage::ActionCommand(ActionCommand::ScrollTo { x: 0, y: 500 });
        let serialized = serde_json::to_string(&cmd).unwrap();
        assert_eq!(
            serialized,
            r#"{"type":"ActionCommand","data":{"type":"scroll_to","x":0,"y":500}}"#
        );
    }

    #[test]
    fn test_action_result_serialization() {
        let res = WsMessage::ActionResult(ActionResult {
            success: true,
            error: None,
            data: None,
        });
        let serialized = serde_json::to_string(&res).unwrap();
        assert_eq!(
            serialized,
            r#"{"type":"ActionResult","data":{"success":true,"error":null,"data":null}}"#
        );
    }
}
