use axum::{
    extract::{Json, State},
    response::{sse::{Event, Sse}, IntoResponse},
};
use async_stream::stream;
use std::convert::Infallible;
use std::sync::Arc;

use crate::dtos::AgentRequest;
use crate::state::AppState;
use crate::models::ChatResponse;

pub async fn run_agent(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<AgentRequest>,
) -> impl IntoResponse {
    tracing::info!("Agent request: {}", request.query);
    
    if request.stream {
        // Return SSE stream
        let stream = stream! {
            yield Ok::<_, Infallible>(Event::default().data("Starting..."));
            // TODO: Integrate with LLM provider for streaming
            yield Ok(Event::default().data("Response chunk 1"));
            yield Ok(Event::default().data("Response chunk 2"));
            yield Ok(Event::default().data("[DONE]"));
        };
        Sse::new(stream).into_response()
    } else {
        // Return JSON
        Json(ChatResponse {
            response: "Non-streaming response placeholder".to_string(),
            prompt_tokens: None,
            response_tokens: None,
            total_tokens: None,
        }).into_response()
    }
}
