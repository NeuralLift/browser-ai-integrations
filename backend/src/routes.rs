use crate::handler::agent_handler;
use crate::models::ws::{ActionCommand, WsMessage};
use crate::state::AppState;
use axum::{
    Router,
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
    routing::{get, post},
};
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::mpsc;
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

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

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let session_id = Uuid::new_v4().to_string();
    tracing::info!("New WebSocket connection: session_id={}", session_id);

    let (mut sink, mut stream) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<WsMessage>();

    // Register connection
    state
        .register_connection(session_id.clone(), tx.clone())
        .await;

    // Send session_id to frontend
    let init_msg = WsMessage::SessionInit {
        session_id: session_id.clone(),
    };
    let _ = tx.send(init_msg);
    tracing::info!("Sent session_init to client");

    // Spawn task to forward messages from channel to WebSocket
    let session_id_clone = session_id.clone();
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Ok(text) = serde_json::to_string(&msg)
                && sink.send(Message::Text(text.into())).await.is_err()
            {
                break;
            }
        }
        tracing::info!("Send task terminated for session_id={}", session_id_clone);
    });

    while let Some(msg) = stream.next().await {
        if let Ok(Message::Text(text)) = msg {
            match serde_json::from_str::<WsMessage>(&text) {
                Ok(WsMessage::Ping) => {
                    let _ = tx.send(WsMessage::Pong);
                }
                Ok(WsMessage::SessionUpdate { url, title }) => {
                    tracing::info!("Context update: url={}, title={:?}", url, title);
                }
                Ok(WsMessage::ActionRequest {
                    request_id,
                    command,
                }) => {
                    match &command {
                        ActionCommand::NavigateTo { url } => {
                            tracing::info!(
                                "ActionRequest[{}]: navigate_to url={}",
                                request_id,
                                url
                            );
                        }
                        ActionCommand::ClickElement { ref_id } => {
                            tracing::info!(
                                "ActionRequest[{}]: click_element ref={}",
                                request_id,
                                ref_id
                            );
                        }
                        ActionCommand::TypeText { ref_id, text } => {
                            tracing::info!(
                                "ActionRequest[{}]: type_text ref={}, text={}",
                                request_id,
                                ref_id,
                                text
                            );
                        }
                        ActionCommand::ScrollTo { x, y } => {
                            tracing::info!(
                                "ActionRequest[{}]: scroll_to x={}, y={}",
                                request_id,
                                x,
                                y
                            );
                        }
                        ActionCommand::GetPageContent { max_length } => {
                            tracing::info!(
                                "ActionRequest[{}]: get_page_content max_length={:?}",
                                request_id,
                                max_length
                            );
                        }
                        ActionCommand::GetInteractiveElements { limit } => {
                            tracing::info!(
                                "ActionRequest[{}]: get_interactive_elements limit={:?}",
                                request_id,
                                limit
                            );
                        }
                    }
                    // NOTE: ActionRequest FROM the client is unusual in this architecture.
                    // The backend sends ActionRequest TO the client (via tools), and the client
                    // sends ActionResult back. This handler is for when the client echoes an
                    // ActionRequest (which shouldn't happen in normal flow).
                    // DO NOT echo back - wait for the real ActionResult from frontend.
                    tracing::warn!(
                        "Received ActionRequest from client (unexpected): {}",
                        request_id
                    );
                }
                Ok(WsMessage::ActionResult(res)) => {
                    tracing::info!(
                        "ActionResult received[{}]: success={}, error={:?}, data={:?}",
                        res.request_id,
                        res.success,
                        res.error,
                        res.data
                    );
                    let request_id = res.request_id.clone();
                    state.complete_pending_action(&request_id, res).await;
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

    // Cleanup
    send_task.abort();
    state.unregister_connection(&session_id).await;
    tracing::info!("WebSocket disconnected: session_id={}", session_id);
}
