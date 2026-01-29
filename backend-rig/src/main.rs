use axum::extract::{Json, State};
use std::net::SocketAddr;
use std::sync::Arc;


mod agent;
mod config;
mod dtos;
mod error;
mod handler;
mod llm;
mod models;
mod routes;
mod state;
mod tools;
mod utils;

use crate::models::{ChatRequest, ChatResponse};
#[cfg(test)]
use crate::models::HealthResponse;
#[cfg(test)]
use rig::message::ImageMediaType;

use crate::state::AppState;


pub async fn chat_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ChatRequest>,
) -> Json<ChatResponse> {
    tracing::info!("Chat request received: {}", request.message);

    match state.llm.complete(
        &request.message,
        request.custom_instruction.as_deref(),
        request.image.as_deref()
    ).await {
        Ok(response) => Json(ChatResponse {
            response,
            prompt_tokens: None,
            response_tokens: None,
            total_tokens: None,
        }),
        Err(e) => {
            tracing::error!("LLM error: {}", e);
            Json(ChatResponse {
                response: format!("Error from AI service: {}", e),
                prompt_tokens: None,
                response_tokens: None,
                total_tokens: None,
            })
        }
    }
}




#[tokio::main]
async fn main() {
    // Load config
    let config = config::AppConfig::from_env();

    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create shared state
    let state = Arc::new(AppState::new());

    // Build the router
    let app = routes::app_router(state);

    // Bind to port
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use crate::llm::parse_image_data;

    #[test]
    fn test_health_response_serialize() {
        let resp = HealthResponse {
            status: "ok".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert_eq!(json, r#"{"status":"ok"}"#);
    }

    #[test]
    fn test_chat_request_deserialize() {
        let json = r#"{
            "message": "Hello",
            "custom_instruction": "Be concise",
            "image": "base64data"
        }"#;
        let req: ChatRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.message, "Hello");
        assert_eq!(req.custom_instruction, Some("Be concise".to_string()));
        assert_eq!(req.image, Some("base64data".to_string()));
    }

    #[test]
    fn test_chat_response_serialize() {
        let resp = ChatResponse {
            response: "Hi".to_string(),
            prompt_tokens: None,
            response_tokens: None,
            total_tokens: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        // Should not contain tokens since they are None and marked with skip_serializing_if
        assert_eq!(json, r#"{"response":"Hi"}"#);

        let resp_with_tokens = ChatResponse {
            response: "Hi".to_string(),
            prompt_tokens: Some(10),
            response_tokens: Some(20),
            total_tokens: Some(30),
        };
        let json_with_tokens = serde_json::to_string(&resp_with_tokens).unwrap();
        assert!(json_with_tokens.contains(r#""prompt_tokens":10"#));
        assert!(json_with_tokens.contains(r#""response_tokens":20"#));
        assert!(json_with_tokens.contains(r#""total_tokens":30"#));
    }

    #[test]
    fn test_base64_prefix_stripping() {
        let png = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAA...";
        let (media_type, data) = parse_image_data(png);
        assert!(matches!(media_type, ImageMediaType::PNG));
        assert_eq!(data, "iVBORw0KGgoAAAANSUhEUgAA...");

        let jpeg = "data:image/jpeg;base64,/9j/4AAQSkZJRgABAQAAAQABAAD...";
        let (media_type, data) = parse_image_data(jpeg);
        assert!(matches!(media_type, ImageMediaType::JPEG));
        assert_eq!(data, "/9j/4AAQSkZJRgABAQAAAQABAAD...");

        let webp = "data:image/webp;base64,UklGRtAAAABXRUJQVlA4...";
        let (media_type, data) = parse_image_data(webp);
        assert!(matches!(media_type, ImageMediaType::WEBP));
        assert_eq!(data, "UklGRtAAAABXRUJQVlA4...");

        let unknown_with_comma = "image/tiff,somebase64data";
        let (media_type, data) = parse_image_data(unknown_with_comma);
        assert!(matches!(media_type, ImageMediaType::JPEG));
        assert_eq!(data, "somebase64data");

        let raw_data = "somebase64datawithoutcomma";
        let (media_type, data) = parse_image_data(raw_data);
        assert!(matches!(media_type, ImageMediaType::JPEG));
        assert_eq!(data, "somebase64datawithoutcomma");
    }
}
