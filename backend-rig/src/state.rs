use crate::llm::GeminiProvider;
use rig::client::ProviderClient;
use rig::providers::gemini;

pub struct AppState {
    pub llm: GeminiProvider,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            llm: GeminiProvider::new(gemini::Client::from_env()),
        }
    }
}
