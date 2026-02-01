use std::env;

pub struct AppConfig {
    pub port: u16,
}

impl AppConfig {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        // Validate that GEMINI_API_KEY is set (required by rig gemini client)
        if env::var("GEMINI_API_KEY").is_err() {
            panic!("GEMINI_API_KEY environment variable is required");
        }

        Self {
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .unwrap(),
        }
    }
}
