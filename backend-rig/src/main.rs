use axum::{
    Router,
    extract::{Json, State},
    routing::{get, post},
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

mod models;
use crate::models::{ChatRequest, ChatResponse, HealthResponse};

use rig::completion::Prompt;
use rig::message::{ImageMediaType, Message, UserContent};
use rig::prelude::*;
use rig::providers::gemini;
use rig::OneOrMany;

struct AppState {
    gemini_client: gemini::Client,
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}

async fn chat_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ChatRequest>,
) -> Json<ChatResponse> {
    tracing::info!("Chat request received: {}", request.message);

    let mut preamble =
        "WAJIB: Selalu jawab dalam Bahasa Indonesia kecuali diminta lain.".to_string();
    if let Some(instruction) = &request.custom_instruction {
        preamble.push_str(&format!("\n\nINSTRUKSI TAMBAHAN: {}", instruction));
    }

    // Initialize agent with model and preamble
    let agent = state
        .gemini_client
        .agent(gemini::completion::GEMINI_2_5_FLASH)
        .preamble(&preamble)
        .build();

    // Prepare content parts
    let mut parts = vec![UserContent::text(request.message.clone())];

    // Add image if present
    if let Some(img_data) = &request.image {
        tracing::info!("Processing image from request");
        let (media_type, data) = parse_image_data(img_data);
        parts.push(UserContent::image_base64(data, Some(media_type), None));
    }

    let prompt = Message::User {
        content: OneOrMany::many(parts).expect("Parts list is not empty"),
    };

    // Prompt the agent and handle the response
    match agent.prompt(prompt).await {
        Ok(response) => Json(ChatResponse {
            response,
            prompt_tokens: None,
            response_tokens: None,
            total_tokens: None,
        }),
        Err(e) => {
            tracing::error!("Rig error: {}", e);
            Json(ChatResponse {
                response: format!("Error from AI service: {}", e),
                prompt_tokens: None,
                response_tokens: None,
                total_tokens: None,
            })
        }
    }
}

fn parse_image_data(img_data: &str) -> (ImageMediaType, &str) {
    if let Some(stripped) = img_data.strip_prefix("data:image/png;base64,") {
        (ImageMediaType::PNG, stripped)
    } else if let Some(stripped) = img_data.strip_prefix("data:image/jpeg;base64,") {
        (ImageMediaType::JPEG, stripped)
    } else if let Some(stripped) = img_data.strip_prefix("data:image/webp;base64,") {
        (ImageMediaType::WEBP, stripped)
    } else {
        if let Some(comma_pos) = img_data.find(',') {
            (ImageMediaType::JPEG, &img_data[comma_pos + 1..])
        } else {
            (ImageMediaType::JPEG, img_data)
        }
    }
}

#[tokio::main]
async fn main() {
    // Load .env file if exists
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Initialize Gemini client from environment
    let gemini_client = gemini::Client::from_env();

    // Create shared state
    let state = Arc::new(AppState { gemini_client });

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build the router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/chat", post(chat_handler))
        .with_state(state)
        .layer(cors);

    // Bind to port 3000
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

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
