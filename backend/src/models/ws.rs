use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum WsMessage {
    Ping,
    Pong,
    #[serde(rename = "session_init")]
    SessionInit {
        session_id: String,
    },
    SessionUpdate {
        url: String,
        title: Option<String>,
    },
    #[serde(rename = "action_request")]
    ActionRequest {
        request_id: String,
        command: ActionCommand,
    },
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
    #[serde(rename = "get_page_content")]
    GetPageContent { max_length: Option<usize> },
    #[serde(rename = "get_interactive_elements")]
    GetInteractiveElements { limit: Option<usize> },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ActionResult {
    pub request_id: String,
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
        let cmd = WsMessage::ActionRequest {
            request_id: "123".to_string(),
            command: ActionCommand::ClickElement { ref_id: 1 },
        };
        let serialized = serde_json::to_string(&cmd).unwrap();
        assert_eq!(
            serialized,
            r#"{"type":"action_request","data":{"request_id":"123","command":{"type":"click_element","ref":1}}}"#
        );

        let cmd = WsMessage::ActionRequest {
            request_id: "123".to_string(),
            command: ActionCommand::NavigateTo {
                url: "https://example.com".to_string(),
            },
        };
        let serialized = serde_json::to_string(&cmd).unwrap();
        assert_eq!(
            serialized,
            r#"{"type":"action_request","data":{"request_id":"123","command":{"type":"navigate_to","url":"https://example.com"}}}"#
        );
    }

    #[test]
    fn test_action_result_serialization() {
        let res = WsMessage::ActionResult(ActionResult {
            request_id: "123".to_string(),
            success: true,
            error: None,
            data: None,
        });
        let serialized = serde_json::to_string(&res).unwrap();
        assert_eq!(
            serialized,
            r#"{"type":"ActionResult","data":{"request_id":"123","success":true,"error":null,"data":null}}"#
        );
    }
}
