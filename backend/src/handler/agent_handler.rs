use axum::{
    extract::{Json, State},
    response::{sse::{Event, Sse}, IntoResponse},
    http::StatusCode,
};
use async_stream::stream;
use futures::StreamExt;
use std::sync::Arc;
use rig::tool::Tool;
use rig::completion::{ToolDefinition, PromptError, Prompt}; // Changed Chat to Prompt
use rig::providers::gemini;
use rig::client::{CompletionClient, ProviderClient}; // Added both
use tokio::sync::oneshot;
use tokio::time::{timeout, Duration};
use uuid::Uuid;

use crate::dtos::AgentRequest;
use crate::state::AppState;
use crate::models::ChatResponse;
use crate::models::ws::{ActionCommand, ActionResult, WsMessage};
use crate::tools::browser::{NavigateTool, ClickTool, TypeTool, ScrollTool, NavigateArgs, ClickArgs, TypeArgs, ScrollArgs};

// --- Error Type ---
#[derive(Debug)]
struct ToolError(String);

impl std::fmt::Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ToolError {}

// --- Tool Implementations ---

struct WsNavigateTool {
    state: Arc<AppState>,
    session_id: String,
}

impl Tool for WsNavigateTool {
    const NAME: &'static str = NavigateTool::NAME;
    type Error = ToolError; // Changed from String
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

struct WsClickTool {
    state: Arc<AppState>,
    session_id: String,
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
        execute_tool(&self.state, &self.session_id, ActionCommand::ClickElement { ref_id: args.ref_id })
            .await
            .map_err(ToolError)
    }
}

struct WsTypeTool {
    state: Arc<AppState>,
    session_id: String,
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
        execute_tool(&self.state, &self.session_id, ActionCommand::TypeText { ref_id: args.ref_id, text: args.text })
            .await
            .map_err(ToolError)
    }
}

struct WsScrollTool {
    state: Arc<AppState>,
    session_id: String,
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
        execute_tool(&self.state, &self.session_id, ActionCommand::ScrollTo { x: args.x, y: args.y })
            .await
            .map_err(ToolError)
    }
}

async fn execute_tool(state: &Arc<AppState>, session_id: &str, command: ActionCommand) -> Result<String, String> {
    // 1. Get connection
    let tx = state.get_connection(session_id).await.ok_or("No active WebSocket connection for this session")?;
    
    // 2. Register pending action
    let request_id = Uuid::new_v4().to_string();
    let (tx_result, rx_result) = oneshot::channel();
    state.register_pending_action(request_id.clone(), tx_result).await;
    
    // 3. Send command
    let msg = WsMessage::ActionRequest {
        request_id: request_id.clone(),
        command,
    };
    
    tx.send(msg).map_err(|e| format!("Failed to send WebSocket message: {}", e))?;
    tracing::info!("Sent ActionRequest[{}] to session {}", request_id, session_id);

    // 4. Wait for result
    let result = timeout(Duration::from_secs(30), rx_result).await
        .map_err(|_| "Tool execution timed out after 30 seconds")?
        .map_err(|_| "Response channel closed unexpectedly")?;
        
    // 5. Return result
    if result.success {
        Ok(format!("Success. Data: {:?}", result.data))
    } else {
        Err(format!("Error: {:?}", result.error))
    }
}

// --- Main Handler ---

pub async fn run_agent(
    State(state): State<Arc<AppState>>,
    Json(request): Json<AgentRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    tracing::info!("Agent request: {} (session_id: {:?})", request.query, request.session_id);
    
    // If session_id is provided, use the tool-enabled agent
    if let Some(session_id) = &request.session_id {
        tracing::info!("Using tool-enabled agent with session_id: {}", session_id);
        // Note: For now, we only support non-streaming tool use because Rig's streaming with tools is complex
        // and needs careful event handling.
        
        let client = gemini::Client::from_env(); // Create fresh client to build agent
        
        let agent = client.agent(gemini::completion::GEMINI_2_5_FLASH)
            .preamble("You are a browser assistant. You can control the browser using tools. Always use tools to answer if possible.")
            .tool(WsNavigateTool { state: state.clone(), session_id: session_id.clone() })
            .tool(WsClickTool { state: state.clone(), session_id: session_id.clone() })
            .tool(WsTypeTool { state: state.clone(), session_id: session_id.clone() })
            .tool(WsScrollTool { state: state.clone(), session_id: session_id.clone() })
            .build();
            
        let response: String = agent.prompt(&request.query)
            .await
            .map_err(|e: PromptError| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            
        Ok(Json(ChatResponse {
            response,
            prompt_tokens: None,
            response_tokens: None,
            total_tokens: None,
        }).into_response())
        
    } else {
        // Legacy path (no tools, just chat)
        if request.stream {
            // Return SSE stream
            let llm_stream = state.llm.stream(
                &request.query,
                request.custom_instruction.as_deref(),
                request.image.as_deref(),
            );
            
            let stream = stream! {
                let mut llm_stream = llm_stream;
                while let Some(chunk) = llm_stream.next().await {
                    match chunk {
                        Ok(text) => yield Ok::<_, String>(Event::default().data(text)),
                        Err(e) => yield Ok::<_, String>(Event::default().event("error").data(e)),
                    }
                }
                yield Ok::<_, String>(Event::default().data("[DONE]"));
            };
            
            Ok(Sse::new(stream).into_response())
        } else {
            // Return JSON
            let response = state.llm.complete(
                &request.query,
                request.custom_instruction.as_deref(),
                request.image.as_deref(),
            )
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

            Ok(Json(ChatResponse {
                response,
                prompt_tokens: None,
                response_tokens: None,
                total_tokens: None,
            }).into_response())
        }
    }
}
