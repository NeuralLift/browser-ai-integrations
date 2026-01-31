use crate::llm::GeminiProvider;
use crate::models::ws::{ActionResult, WsMessage};
use rig::client::ProviderClient;
use rig::providers::gemini;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, RwLock};

pub struct AppState {
    pub llm: GeminiProvider,
    pub active_connections: Arc<RwLock<HashMap<String, mpsc::UnboundedSender<WsMessage>>>>,
    pub pending_actions: Arc<RwLock<HashMap<String, oneshot::Sender<ActionResult>>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            llm: GeminiProvider::new(gemini::Client::from_env()),
            active_connections: Arc::new(RwLock::new(HashMap::new())),
            pending_actions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_connection(
        &self,
        session_id: String,
        sender: mpsc::UnboundedSender<WsMessage>,
    ) {
        let mut connections = self.active_connections.write().await;
        connections.insert(session_id, sender);
    }

    pub async fn unregister_connection(&self, session_id: &str) {
        let mut connections = self.active_connections.write().await;
        connections.remove(session_id);
    }

    pub async fn get_connection(&self, session_id: &str) -> Option<mpsc::UnboundedSender<WsMessage>> {
        let connections = self.active_connections.read().await;
        connections.get(session_id).cloned()
    }

    pub async fn register_pending_action(
        &self,
        request_id: String,
        sender: oneshot::Sender<ActionResult>,
    ) {
        let mut pending = self.pending_actions.write().await;
        pending.insert(request_id, sender);
    }

    pub async fn complete_pending_action(&self, request_id: &str, result: ActionResult) -> bool {
        let mut pending = self.pending_actions.write().await;
        if let Some(sender) = pending.remove(request_id) {
            sender.send(result).is_ok()
        } else {
            false
        }
    }
}

