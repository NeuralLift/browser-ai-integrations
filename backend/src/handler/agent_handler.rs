use axum::{
    extract::{Json, State},
    response::{sse::{Event, Sse}, IntoResponse},
    http::StatusCode,
};
use async_stream::stream;
use futures::StreamExt;
use std::sync::Arc;

use crate::dtos::AgentRequest;
use crate::state::AppState;
use crate::models::ChatResponse;

pub async fn run_agent(
    State(state): State<Arc<AppState>>,
    Json(request): Json<AgentRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    tracing::info!("Agent request: {}", request.query);
    
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
