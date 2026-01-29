use rig::prelude::*;
use rig::providers::gemini;

pub struct AppState {
    pub gemini_client: gemini::Client,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            gemini_client: gemini::Client::from_env(),
        }
    }
}
