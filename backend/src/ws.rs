use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextUpdate {
    #[serde(rename = "type")]
    pub update_type: String,
    pub timestamp: String,
    pub url: Option<String>,
    pub title: Option<String>,
    pub content: Option<String>,
    pub screenshot: Option<String>,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();

    tracing::info!("New WebSocket connection established");

    // Send welcome message
    let welcome = serde_json::json!({
        "type": "connected",
        "message": "Connected to Browser AI Assistant backend"
    });

    if let Err(e) = sender.send(Message::Text(welcome.to_string().into())).await {
        tracing::error!("Error sending welcome message: {}", e);
        return;
    }

    // Handle incoming messages
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                tracing::debug!("Received text message: {} bytes", text.len());

                match serde_json::from_str::<ContextUpdate>(&text) {
                    Ok(context) => {
                        tracing::info!(
                            "Context update received - URL: {:?}, Title: {:?}",
                            context.url,
                            context.title
                        );

                        // Store the latest context
                        let mut current_context = state.current_context.write().await;
                        *current_context = Some(context);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse context update: {}", e);
                    }
                }
            }
            Ok(Message::Close(_)) => {
                tracing::info!("WebSocket connection closed by client");
                break;
            }
            Ok(Message::Ping(data)) => {
                if let Err(e) = sender.send(Message::Pong(data)).await {
                    tracing::error!("Error sending pong: {}", e);
                    break;
                }
            }
            Err(e) => {
                tracing::error!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }

    tracing::info!("WebSocket connection terminated");
}
