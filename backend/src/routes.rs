use axum::{
    Router,
    extract::{State, ws::{WebSocket, WebSocketUpgrade, Message}},
    response::IntoResponse,
    routing::{get, post},
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use crate::state::AppState;
use crate::handler::agent_handler;
use crate::models::ws::{ActionCommand, ActionResult, WsMessage};

pub fn app_router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    
    Router::new()
        .route("/health", get(health_check))
        .route("/api/chat", post(agent_handler::run_agent))
        .route("/agent/run", post(agent_handler::run_agent))
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
    while let Some(msg) = socket.recv().await {
        if let Ok(Message::Text(text)) = msg {
            match serde_json::from_str::<WsMessage>(&text) {
                Ok(WsMessage::Ping) => {
                    let pong = serde_json::to_string(&WsMessage::Pong).unwrap();
                    if socket.send(Message::Text(pong.into())).await.is_err() {
                        break;
                    }
                }
                Ok(WsMessage::SessionUpdate { url, title }) => {
                    tracing::info!("Context update: url={}, title={:?}", url, title);
                }
                Ok(WsMessage::ActionCommand(cmd)) => {
                    match &cmd {
                        ActionCommand::NavigateTo { url } => {
                            tracing::info!("ActionCommand: navigate_to url={}", url);
                        }
                        ActionCommand::ClickElement { ref_id } => {
                            tracing::info!("ActionCommand: click_element ref={}", ref_id);
                        }
                        ActionCommand::TypeText { ref_id, text } => {
                            tracing::info!("ActionCommand: type_text ref={}, text={}", ref_id, text);
                        }
                        ActionCommand::ScrollTo { x, y } => {
                            tracing::info!("ActionCommand: scroll_to x={}, y={}", x, y);
                        }
                    }
                    // Echo back as ActionResult (placeholder for actual execution)
                    let result = WsMessage::ActionResult(ActionResult {
                        success: true,
                        error: None,
                        data: None,
                    });
                    let response = serde_json::to_string(&result).unwrap();
                    if socket.send(Message::Text(response.into())).await.is_err() {
                        break;
                    }
                }
                Ok(WsMessage::ActionResult(res)) => {
                    tracing::info!("ActionResult received: success={}", res.success);
                }
                Ok(WsMessage::Unknown) => {
                    tracing::warn!("Unknown WebSocket message type");
                }
                Err(e) => {
                    tracing::warn!("Failed to parse WebSocket message: {}", e);
                }
                _ => {}
            }
        }
    }
}
