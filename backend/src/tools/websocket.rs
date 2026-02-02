//! WebSocket-based tool implementations for browser automation.
//!
//! These tools wrap the base browser tools and execute them via WebSocket
//! connections to the browser extension.

use std::sync::Arc;
use tokio::sync::oneshot;
use tokio::time::{Duration, timeout};
use uuid::Uuid;

use rig::completion::ToolDefinition;
use rig::tool::Tool;

use crate::models::ws::{ActionCommand, WsMessage};
use crate::state::AppState;
use crate::tools::browser::{
    ClickArgs, ClickTool, GetInteractiveElementsArgs, GetInteractiveElementsTool,
    GetPageContentArgs, GetPageContentTool, NavigateArgs, NavigateTool, ScrollArgs, ScrollTool,
    TypeArgs, TypeTool,
};

// --- Error Type ---
#[derive(Debug)]
pub struct ToolError(pub String);

impl std::fmt::Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ToolError {}

// --- Helper function to execute tools via WebSocket ---
pub(crate) async fn execute_tool(
    state: &Arc<AppState>,
    session_id: &str,
    command: ActionCommand,
) -> Result<String, String> {
    // 1. Get connection
    let tx = state
        .get_connection(session_id)
        .await
        .ok_or("No active WebSocket connection for this session")?;

    // 2. Register pending action
    let request_id = Uuid::new_v4().to_string();
    let (tx_result, rx_result) = oneshot::channel();
    state
        .register_pending_action(request_id.clone(), tx_result)
        .await;

    // 3. Send command
    let msg = WsMessage::ActionRequest {
        request_id: request_id.clone(),
        command,
    };

    tx.send(msg)
        .map_err(|e| format!("Failed to send WebSocket message: {}", e))?;
    tracing::info!(
        "Sent ActionRequest[{}] to session {}",
        request_id,
        session_id
    );

    // 4. Wait for result
    let result = timeout(Duration::from_secs(30), rx_result)
        .await
        .map_err(|_| "Tool execution timed out after 30 seconds")?
        .map_err(|_| "Response channel closed unexpectedly")?;

    // 5. Return result
    if result.success {
        Ok(format!("Success. Data: {:?}", result.data))
    } else {
        Err(format!("Error: {:?}", result.error))
    }
}

// --- Tool Implementations with constructors ---

pub struct WsNavigateTool {
    state: Arc<AppState>,
    session_id: String,
}

impl WsNavigateTool {
    pub fn new(state: Arc<AppState>, session_id: String) -> Self {
        Self { state, session_id }
    }
}

impl Tool for WsNavigateTool {
    const NAME: &'static str = NavigateTool::NAME;
    type Error = ToolError;
    type Args = NavigateArgs;
    type Output = String;

    async fn definition(&self, prompt: String) -> ToolDefinition {
        NavigateTool.definition(prompt).await
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Validate URL - reject system/restricted URLs
        let url_lower = args.url.to_lowercase();
        if url_lower.starts_with("chrome://")
            || url_lower.starts_with("about:")
            || url_lower.starts_with("file://")
        {
            return Err(ToolError(
                "Navigation to system pages (chrome://, about://, file://) is not allowed".into(),
            ));
        }

        execute_tool(
            &self.state,
            &self.session_id,
            ActionCommand::NavigateTo { url: args.url },
        )
        .await
        .map_err(ToolError)
    }
}

pub struct WsClickTool {
    state: Arc<AppState>,
    session_id: String,
}

impl WsClickTool {
    pub fn new(state: Arc<AppState>, session_id: String) -> Self {
        Self { state, session_id }
    }
}

impl Tool for WsClickTool {
    const NAME: &'static str = ClickTool::NAME;
    type Error = ToolError;
    type Args = ClickArgs;
    type Output = String;

    async fn definition(&self, prompt: String) -> ToolDefinition {
        ClickTool.definition(prompt).await
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        execute_tool(
            &self.state,
            &self.session_id,
            ActionCommand::ClickElement {
                ref_id: args.ref_id,
            },
        )
        .await
        .map_err(ToolError)
    }
}

pub struct WsTypeTool {
    state: Arc<AppState>,
    session_id: String,
}

impl WsTypeTool {
    pub fn new(state: Arc<AppState>, session_id: String) -> Self {
        Self { state, session_id }
    }
}

impl Tool for WsTypeTool {
    const NAME: &'static str = TypeTool::NAME;
    type Error = ToolError;
    type Args = TypeArgs;
    type Output = String;

    async fn definition(&self, prompt: String) -> ToolDefinition {
        TypeTool.definition(prompt).await
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        execute_tool(
            &self.state,
            &self.session_id,
            ActionCommand::TypeText {
                ref_id: args.ref_id,
                text: args.text,
            },
        )
        .await
        .map_err(ToolError)
    }
}

pub struct WsScrollTool {
    state: Arc<AppState>,
    session_id: String,
}

impl WsScrollTool {
    pub fn new(state: Arc<AppState>, session_id: String) -> Self {
        Self { state, session_id }
    }
}

impl Tool for WsScrollTool {
    const NAME: &'static str = ScrollTool::NAME;
    type Error = ToolError;
    type Args = ScrollArgs;
    type Output = String;

    async fn definition(&self, prompt: String) -> ToolDefinition {
        ScrollTool.definition(prompt).await
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        execute_tool(
            &self.state,
            &self.session_id,
            ActionCommand::ScrollTo {
                x: args.x,
                y: args.y,
            },
        )
        .await
        .map_err(ToolError)
    }
}

pub struct WsGetPageContentTool {
    state: Arc<AppState>,
    session_id: String,
}

impl WsGetPageContentTool {
    pub fn new(state: Arc<AppState>, session_id: String) -> Self {
        Self { state, session_id }
    }
}

impl Tool for WsGetPageContentTool {
    const NAME: &'static str = GetPageContentTool::NAME;
    type Error = ToolError;
    type Args = GetPageContentArgs;
    type Output = String;

    async fn definition(&self, prompt: String) -> ToolDefinition {
        GetPageContentTool.definition(prompt).await
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        execute_tool(
            &self.state,
            &self.session_id,
            ActionCommand::GetPageContent {
                max_length: args.max_length,
            },
        )
        .await
        .map_err(ToolError)
    }
}

pub struct WsGetInteractiveElementsTool {
    state: Arc<AppState>,
    session_id: String,
}

impl WsGetInteractiveElementsTool {
    pub fn new(state: Arc<AppState>, session_id: String) -> Self {
        Self { state, session_id }
    }
}

impl Tool for WsGetInteractiveElementsTool {
    const NAME: &'static str = GetInteractiveElementsTool::NAME;
    type Error = ToolError;
    type Args = GetInteractiveElementsArgs;
    type Output = String;

    async fn definition(&self, prompt: String) -> ToolDefinition {
        GetInteractiveElementsTool.definition(prompt).await
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        execute_tool(
            &self.state,
            &self.session_id,
            ActionCommand::GetInteractiveElements { limit: args.limit },
        )
        .await
        .map_err(ToolError)
    }
}
