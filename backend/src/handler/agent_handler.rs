use async_stream::stream;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{
        IntoResponse,
        sse::{Event, Sse},
    },
};
use futures::StreamExt;
use rig::OneOrMany;
use rig::client::{CompletionClient, ProviderClient}; // Added both
use rig::completion::{Prompt, ToolDefinition};
use rig::message::{ImageMediaType, Message, UserContent};
use rig::providers::gemini;
use rig::tool::Tool;
use std::sync::Arc;
use tokio::sync::oneshot;
use tokio::time::{Duration, timeout};
use uuid::Uuid;

use crate::dtos::AgentRequest;
use crate::models::ChatResponse;
use crate::models::ws::{ActionCommand, WsMessage};
use crate::state::AppState;
use crate::tools::browser::{
    ClickArgs, ClickTool, GetInteractiveElementsArgs, GetInteractiveElementsTool,
    GetPageContentArgs, GetPageContentTool, NavigateArgs, NavigateTool, ScrollArgs, ScrollTool,
    TypeArgs, TypeTool,
};

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

struct WsGetPageContentTool {
    state: Arc<AppState>,
    session_id: String,
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

struct WsGetInteractiveElementsTool {
    state: Arc<AppState>,
    session_id: String,
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

async fn execute_tool(
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

// --- Main Handler ---

pub async fn run_agent(
    State(state): State<Arc<AppState>>,
    Json(request): Json<AgentRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    tracing::info!(
        "Agent request: {} (session_id: {:?})",
        request.query,
        request.session_id
    );

    // If session_id is provided, use the tool-enabled agent
    if let Some(session_id) = &request.session_id {
        tracing::info!("Using tool-enabled agent with session_id: {}", session_id);
        // Note: For now, we only support non-streaming tool use because Rig's streaming with tools is complex
        // and needs careful event handling.

        let client = gemini::Client::from_env(); // Create fresh client to build agent

        let preamble = r#"You are a browser automation assistant. You can control the browser using tools AND see/analyze screenshots.

## Available Tools
### Action Tools
- `navigate_to(url)`: Navigate to a URL (e.g., "https://google.com")
- `click_element(ref)`: Click an element using its Ref ID number
- `type_text(ref, text)`: Type text into an input field using its Ref ID
- `scroll_to(x, y)`: Scroll the page to coordinates

### Context Tools (use these FIRST when needed)
- `get_interactive_elements(limit)`: Scan page for buttons, inputs, links. **CALL THIS FIRST** before clicking or typing.
- `get_page_content(max_length)`: Get page text content. Use when you need to read, summarize, or analyze text.

## Your Capabilities
1. **Browser Automation**: Control the browser using action tools
2. **Visual Analysis**: When screenshot is provided, you CAN SEE and READ everything visible on screen
3. **Dynamic Context**: Use context tools to get page data when needed

## Instructions
1. **Before clicking/typing**: Call `get_interactive_elements()` to find element Ref IDs
2. **Before reading/summarizing**: Call `get_page_content()` to get page text
3. When the user asks to go to a website, use `navigate_to`
4. When the user asks about the page content (with screenshot), read the screenshot OR call `get_page_content()`
5. Always respond with a brief confirmation of what you did

## Example Flows
- User: "klik tombol login" → Call get_interactive_elements() → Find login button Ref ID → Call click_element(ref)
- User: "rangkum halaman ini" → Call get_page_content() → Summarize the returned text
- User: "buka google" → Call navigate_to("https://google.com")
"#.to_string();

        let agent = client
            .agent(gemini::completion::GEMINI_2_5_FLASH)
            .preamble(&preamble)
            .tool(WsNavigateTool {
                state: state.clone(),
                session_id: session_id.clone(),
            })
            .tool(WsClickTool {
                state: state.clone(),
                session_id: session_id.clone(),
            })
            .tool(WsTypeTool {
                state: state.clone(),
                session_id: session_id.clone(),
            })
            .tool(WsScrollTool {
                state: state.clone(),
                session_id: session_id.clone(),
            })
            .tool(WsGetPageContentTool {
                state: state.clone(),
                session_id: session_id.clone(),
            })
            .tool(WsGetInteractiveElementsTool {
                state: state.clone(),
                session_id: session_id.clone(),
            })
            .default_max_depth(20) // Allow up to 20 tool call rounds for complex actions
            .build();

        // Build the prompt - either text-only or text+image
        let response: String = if let Some(image_data) = &request.image {
            // Strip data URL prefix if present (e.g., "data:image/jpeg;base64,")
            let base64_data = if let Some(pos) = image_data.find(",") {
                &image_data[pos + 1..]
            } else {
                image_data.as_str()
            };

            // Build multimodal message with text + image
            let mut content_parts = vec![UserContent::text(&request.query)];
            content_parts.push(UserContent::image_base64(
                base64_data,
                Some(ImageMediaType::JPEG),
                None,
            ));

            let user_message = Message::User {
                content: OneOrMany::many(content_parts).unwrap(),
            };

            tracing::info!("Sending multimodal prompt (text + image) to agent");
            match agent.prompt(user_message).await {
                Ok(text) => text,
                Err(e) => {
                    let error_str = e.to_string();
                    tracing::warn!("Agent multimodal prompt error: {}", error_str);
                    if error_str.contains("empty") || error_str.contains("no message") {
                        "Maaf, saya tidak bisa menganalisis gambar ini dalam mode browser automation. Coba matikan fitur Browser Agent untuk analisis gambar.".to_string()
                    } else {
                        return Err((StatusCode::INTERNAL_SERVER_ERROR, error_str));
                    }
                }
            }
        } else {
            // Text-only prompt
            match agent.prompt(&request.query).await {
                Ok(text) => text,
                Err(e) => {
                    let error_str = e.to_string();
                    tracing::warn!("Agent prompt error: {}", error_str);

                    // Handle empty response error gracefully
                    if error_str.contains("empty") || error_str.contains("no message") {
                        "Maaf, saya tidak yakin tindakan apa yang harus dilakukan. Bisa tolong jelaskan lebih spesifik? Contoh:\n- \"isi field email dengan test@example.com\"\n- \"klik tombol Submit\"\n- \"buka halaman google.com\"".to_string()
                    } else if error_str.contains("MaxDepth") || error_str.contains("depth") {
                        // Tool execution failed after retries - likely a connection issue
                        "Maaf, gagal menjalankan aksi browser. Pastikan:\n1. Extension Chrome sudah di-reload\n2. Halaman web sudah terbuka dan aktif\n3. Coba refresh halaman dan ulangi perintah".to_string()
                    } else {
                        return Err((StatusCode::INTERNAL_SERVER_ERROR, error_str));
                    }
                }
            }
        };

        Ok(Json(ChatResponse {
            response,
            prompt_tokens: None,
            response_tokens: None,
            total_tokens: None,
        })
        .into_response())
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
            let response = state
                .llm
                .complete(
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
            })
            .into_response())
        }
    }
}
