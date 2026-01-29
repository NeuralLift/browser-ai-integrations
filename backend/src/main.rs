use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};

mod ai;
mod memory;
mod privacy;
mod ws;

use privacy::sanitize_context;
use ws::ContextUpdate;

#[derive(Clone)]
pub struct AppState {
    pub current_context: Arc<RwLock<Option<ContextUpdate>>>,
    pub memory_pool: SqlitePool,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
}

#[derive(Deserialize)]
struct ChatRequest {
    message: String,
    custom_instruction: Option<String>,
    image: Option<String>,
}

#[derive(Serialize)]
struct ChatResponse {
    response: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    prompt_tokens: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_tokens: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    total_tokens: Option<i32>,
}

#[derive(Deserialize)]
struct CreateMemoryRequest {
    content: String,
}

#[derive(Serialize)]
struct CreateMemoryResponse {
    id: i64,
}

async fn hello_world() -> &'static str {
    "Hello World"
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}

// Debug endpoint to see what context is being captured
async fn debug_context(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let context_guard = state.current_context.read().await;
    match context_guard.as_ref() {
        Some(ctx) => Json(serde_json::json!({
            "has_context": true,
            "url": ctx.url,
            "title": ctx.title,
            "content_length": ctx.content.as_ref().map(|c| c.len()),
            "content_preview": ctx.content.as_ref().map(|c| {
                if c.len() > 500 { format!("{}...", &c[..500]) } else { c.clone() }
            }),
            "has_screenshot": ctx.screenshot.is_some(),
        })),
        None => Json(serde_json::json!({
            "has_context": false,
            "message": "No context received yet. Make sure extension is connected."
        })),
    }
}

async fn chat_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ChatRequest>,
) -> Json<ChatResponse> {
    tracing::info!("Chat request received: {}", request.message);

    // Get current context
    let context_guard = state.current_context.read().await;
    let sanitized = context_guard.as_ref().map(|ctx| sanitize_context(ctx));
    drop(context_guard);

    // Try to get AI response
    let (response_text, usage_metadata) = match ai::AiClient::new() {
        Ok(client) => {
            match client
                .ask(
                    &state.memory_pool,
                    sanitized.as_ref(),
                    &request.message,
                    request.custom_instruction.as_deref(),
                    request.image.as_deref(),
                )
                .await
            {
                Ok((reply, usage)) => (reply, usage),
                Err(e) => {
                    tracing::error!("AI error: {}", e);
                    (
                        format!("AI service error: {}. Make sure GOOGLE_API_KEY is set.", e),
                        None,
                    )
                }
            }
        }
        Err(e) => {
            tracing::warn!("AI client not configured: {}", e);
            // Fallback response when API key is not configured
            let reply = if let Some(ctx) = sanitized {
                format!(
                    "I can see you're on: {}\n\nPage title: {}\n\n(AI integration requires GOOGLE_API_KEY environment variable)",
                    ctx.url.unwrap_or_default(),
                    ctx.title.unwrap_or_default()
                )
            } else {
                "No browser context received yet. Open a webpage and the context will be captured automatically.\n\n(AI integration requires GOOGLE_API_KEY environment variable)".to_string()
            };
            (reply, None)
        }
    };

    let (prompt_tokens, response_tokens, total_tokens) = if let Some(usage) = usage_metadata {
        (
            Some(usage.prompt_token_count),
            usage.candidates_token_count,
            Some(usage.total_token_count),
        )
    } else {
        (None, None, None)
    };

    Json(ChatResponse {
        response: response_text,
        prompt_tokens,
        response_tokens,
        total_tokens,
    })
}

async fn create_memory(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateMemoryRequest>,
) -> Result<Json<CreateMemoryResponse>, StatusCode> {
    tracing::info!("Creating new memory: {}", req.content);
    match memory::add_memory(&state.memory_pool, &req.content).await {
        Ok(id) => Ok(Json(CreateMemoryResponse { id })),
        Err(e) => {
            tracing::error!("Failed to add memory: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn list_memories(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<memory::Memory>>, StatusCode> {
    match memory::get_recent_memories(&state.memory_pool, 50).await {
        Ok(memories) => Ok(Json(memories)),
        Err(e) => {
            tracing::error!("Failed to list memories: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn delete_memory(State(state): State<Arc<AppState>>, Path(id): Path<i64>) -> StatusCode {
    tracing::info!("Deleting memory ID: {}", id);
    match memory::delete_memory(&state.memory_pool, id).await {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            tracing::error!("Failed to delete memory: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

#[tokio::main]
async fn main() {
    // Load .env file if exists
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Debug: check if API key is set
    match std::env::var("GOOGLE_API_KEY") {
        Ok(key) => tracing::info!("GOOGLE_API_KEY is set (length: {})", key.len()),
        Err(_) => tracing::warn!("GOOGLE_API_KEY not found! AI features will be disabled."),
    }

    // Initialize SQLite pool
    let pool = SqlitePool::connect("sqlite:memories.db?mode=rwc")
        .await
        .unwrap();
    memory::init_db(&pool).await.unwrap();

    // Create shared state
    let state = Arc::new(AppState {
        current_context: Arc::new(RwLock::new(None)),
        memory_pool: pool,
    });

    // Configure CORS to allow chrome-extension:// origins
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build the router
    let app = Router::new()
        .route("/", get(hello_world))
        .route("/health", get(health_check))
        .route("/debug/context", get(debug_context))
        .route("/ws", get(ws::ws_handler))
        .route("/api/chat", post(chat_handler))
        .route("/api/memory", get(list_memories).post(create_memory))
        .route("/api/memory/{id}", delete(delete_memory))
        .layer(cors)
        .with_state(state);

    // Bind to port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("Server running on http://localhost:3000");

    axum::serve(listener, app).await.unwrap();
}
