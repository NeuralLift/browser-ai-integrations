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
use rig::agent::MultiTurnStreamItem;
use rig::client::{CompletionClient, ProviderClient};
use rig::completion::GetTokenUsage;
use rig::message::{AssistantContent, ImageMediaType, Message, UserContent};
use rig::streaming::{StreamedAssistantContent, StreamingChat};

use rig::providers::gemini;

use crate::tools::websocket::{
    WsClickTool, WsGetInteractiveElementsTool, WsGetPageContentTool, WsNavigateTool, WsScrollTool,
    WsTypeTool,
};
use std::sync::Arc;

use crate::dtos::AgentRequest;
use crate::models::ChatResponse;
use crate::state::AppState;

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

    // If session_id is provided, use the tool-enabled agent with STREAMING
    if let Some(session_id) = &request.session_id {
        tracing::info!(
            "Using streaming tool-enabled agent with session_id: {}",
            session_id
        );

        // Convert history to Vec<Message>
        let chat_history: Vec<Message> = if let Some(history) = &request.history {
            history
                .iter()
                .map(|msg| match msg.role.as_str() {
                    "user" => Message::User {
                        content: OneOrMany::one(UserContent::text(&msg.content)),
                    },
                    "assistant" => Message::Assistant {
                        id: None,
                        content: OneOrMany::one(AssistantContent::text(&msg.content)),
                    },
                    _ => Message::User {
                        content: OneOrMany::one(UserContent::text(&msg.content)),
                    },
                })
                .collect()
        } else {
            vec![]
        };

        let client = gemini::Client::from_env();

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
            .tool(WsNavigateTool::new(state.clone(), session_id.clone()))
            .tool(WsClickTool::new(state.clone(), session_id.clone()))
            .tool(WsTypeTool::new(state.clone(), session_id.clone()))
            .tool(WsScrollTool::new(state.clone(), session_id.clone()))
            .tool(WsGetPageContentTool::new(state.clone(), session_id.clone()))
            .tool(WsGetInteractiveElementsTool::new(
                state.clone(),
                session_id.clone(),
            ))
            .default_max_depth(20)
            .build();

        // Build the prompt - either text-only or text+image
        let user_message: Message = if let Some(image_data) = &request.image {
            // Strip data URL prefix if present
            let base64_data = if let Some(pos) = image_data.find(",") {
                &image_data[pos + 1..]
            } else {
                image_data.as_str()
            };

            let mut content_parts = vec![UserContent::text(&request.query)];
            content_parts.push(UserContent::image_base64(
                base64_data,
                Some(ImageMediaType::JPEG),
                None,
            ));

            Message::User {
                content: OneOrMany::many(content_parts).unwrap(),
            }
        } else {
            Message::User {
                content: OneOrMany::one(UserContent::text(&request.query)),
            }
        };

        // Use stream_chat for streaming with tools
        let mut agent_stream = agent.stream_chat(user_message, chat_history).await;

        let sse_stream = stream! {
            let mut full_response = String::new();
            let mut token_usage: Option<(u64, u64, u64)> = None;

            while let Some(chunk) = agent_stream.next().await {
                match chunk {
                    Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(text))) => {
                        full_response.push_str(&text.text);
                        yield Ok::<_, String>(Event::default().data(&text.text));
                    }
                    Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::ToolCall(tool_call))) => {
                        // Notify frontend about tool execution
                        let tool_info = format!(r#"{{"__type":"tool","name":"{}","status":"calling"}}"#, tool_call.function.name);
                        yield Ok::<_, String>(Event::default().event("tool").data(tool_info));
                    }
                    Ok(MultiTurnStreamItem::StreamUserItem(_user_content)) => {
                        // Tool result - notify frontend
                        let result_info = r#"{"__type":"tool","status":"completed"}"#;
                        yield Ok::<_, String>(Event::default().event("tool").data(result_info));
                    }
                    Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Final(final_resp))) => {
                        if let Some(usage) = final_resp.token_usage() {
                            token_usage = Some((usage.input_tokens, usage.output_tokens, usage.total_tokens));
                        }
                    }
                    Ok(MultiTurnStreamItem::FinalResponse(final_resp)) => {
                        let usage = final_resp.usage();
                        token_usage = Some((usage.input_tokens, usage.output_tokens, usage.total_tokens));
                    }
                    Ok(_) => {
                        // Other variants (Reasoning, etc.)
                    }
                    Err(e) => {
                        let error_str = e.to_string();
                        tracing::warn!("Agent stream error: {}", error_str);

                        // Handle specific errors gracefully
                        let error_msg = if error_str.contains("empty") || error_str.contains("no message") {
                            "Maaf, saya tidak yakin tindakan apa yang harus dilakukan.".to_string()
                        } else if error_str.contains("MaxDepth") || error_str.contains("depth") {
                            "Maaf, gagal menjalankan aksi browser. Coba refresh halaman.".to_string()
                        } else {
                            format!("Error: {}", error_str)
                        };
                        yield Ok::<_, String>(Event::default().event("error").data(error_msg));
                    }
                }
            }

            // Send token usage at end
            if let Some((input, output, total)) = token_usage {
                let usage_json = format!(
                    r#"{{"__type":"usage","input_tokens":{},"output_tokens":{},"total_tokens":{}}}"#,
                    input, output, total
                );
                yield Ok::<_, String>(Event::default().event("usage").data(usage_json));
            }

            yield Ok::<_, String>(Event::default().data("[DONE]"));
        };

        Ok(Sse::new(sse_stream).into_response())
    } else {
        // Legacy path (no tools, just chat)
        // TODO: Update state.llm.stream/complete to support chat history
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
                        Ok(text) => {
                            // Check if this is usage metadata (sent at end of stream)
                            if text.starts_with(r#"{"__type":"usage""#) {
                                yield Ok::<_, String>(Event::default().event("usage").data(text));
                            } else {
                                yield Ok::<_, String>(Event::default().data(text));
                            }
                        }
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
