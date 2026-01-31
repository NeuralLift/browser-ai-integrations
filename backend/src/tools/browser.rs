use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::error::Error;
use std::fmt;

#[derive(Debug, Serialize, Deserialize)]
pub struct BrowserToolError(String);

impl fmt::Display for BrowserToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Browser tool error: {}", self.0)
    }
}

impl Error for BrowserToolError {}

/// Tool to navigate to a specific URL
#[derive(Deserialize, Serialize)]
pub struct NavigateTool;

#[derive(Deserialize, Serialize)]
pub struct NavigateArgs {
    pub url: String,
}

impl Tool for NavigateTool {
    const NAME: &'static str = "navigate_to";
    type Error = BrowserToolError;
    type Args = NavigateArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Navigate to a specific URL in the browser".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "The URL to navigate to (e.g., https://google.com)"
                    }
                },
                "required": ["url"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        Ok(format!("Navigating to {}", args.url))
    }
}

/// Tool to click an element by its reference ID
#[derive(Deserialize, Serialize)]
pub struct ClickTool;

#[derive(Deserialize, Serialize)]
pub struct ClickArgs {
    #[serde(rename = "ref")]
    pub ref_id: i32,
}

impl Tool for ClickTool {
    const NAME: &'static str = "click_element";
    type Error = BrowserToolError;
    type Args = ClickArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Click an element on the page using its reference ID".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "ref": {
                        "type": "integer",
                        "description": "The reference ID of the element to click"
                    }
                },
                "required": ["ref"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        Ok(format!("Clicking element with ref ID: {}", args.ref_id))
    }
}

/// Tool to type text into an element
#[derive(Deserialize, Serialize)]
pub struct TypeTool;

#[derive(Deserialize, Serialize)]
pub struct TypeArgs {
    #[serde(rename = "ref")]
    pub ref_id: i32,
    pub text: String,
}

impl Tool for TypeTool {
    const NAME: &'static str = "type_text";
    type Error = BrowserToolError;
    type Args = TypeArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Type text into an input field using its reference ID".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "ref": {
                        "type": "integer",
                        "description": "The reference ID of the element to type into"
                    },
                    "text": {
                        "type": "string",
                        "description": "The text to type"
                    }
                },
                "required": ["ref", "text"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        Ok(format!("Typing '{}' into element with ref ID: {}", args.text, args.ref_id))
    }
}

/// Tool to scroll the page
#[derive(Deserialize, Serialize)]
pub struct ScrollTool;

#[derive(Deserialize, Serialize)]
pub struct ScrollArgs {
    pub x: i32,
    pub y: i32,
}

impl Tool for ScrollTool {
    const NAME: &'static str = "scroll_to";
    type Error = BrowserToolError;
    type Args = ScrollArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Scroll the page to specific coordinates (x, y)".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "x": {
                        "type": "integer",
                        "description": "The x-coordinate to scroll to"
                    },
                    "y": {
                        "type": "integer",
                        "description": "The y-coordinate to scroll to"
                    }
                },
                "required": ["x", "y"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        Ok(format!("Scrolling to x: {}, y: {}", args.x, args.y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_navigate_tool_serialization() {
        let args_json = json!({ "url": "https://example.com" });
        let args: NavigateArgs = serde_json::from_value(args_json).unwrap();
        assert_eq!(args.url, "https://example.com");
    }

    #[tokio::test]
    async fn test_click_tool_serialization() {
        let args_json = json!({ "ref": 42 });
        let args: ClickArgs = serde_json::from_value(args_json).unwrap();
        assert_eq!(args.ref_id, 42);
    }

    #[tokio::test]
    async fn test_type_tool_serialization() {
        let args_json = json!({ "ref": 42, "text": "hello" });
        let args: TypeArgs = serde_json::from_value(args_json).unwrap();
        assert_eq!(args.ref_id, 42);
        assert_eq!(args.text, "hello");
    }

    #[tokio::test]
    async fn test_scroll_tool_serialization() {
        let args_json = json!({ "x": 100, "y": 200 });
        let args: ScrollArgs = serde_json::from_value(args_json).unwrap();
        assert_eq!(args.x, 100);
        assert_eq!(args.y, 200);
    }
}
