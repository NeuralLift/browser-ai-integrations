use axum::{
    Router,
    extract::{State, ws::{WebSocket, WebSocketUpgrade}},
    response::IntoResponse,
    routing::{get, post},
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use crate::state::AppState;
// use crate::handler::agent_handler;  // Will be created in Task 6
use crate::chat_handler;

pub fn app_router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    
    Router::new()
        .route("/health", get(health_check))
        .route("/api/chat", post(chat_handler))  // Keep existing for now
        .route("/ws", get(ws_handler))
        .with_state(state)
        .layer(cors)
}

async fn health_check() -> impl IntoResponse {
    axum::Json(serde_json::json!({"status": "ok"}))
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    // Basic WebSocket handler - receives context updates
    while let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            tracing::info!("WebSocket message received: {:?}", msg);
            // Echo back for now - will be enhanced later
            if socket.send(msg).await.is_err() {
                break;
            }
        }
    }
}
