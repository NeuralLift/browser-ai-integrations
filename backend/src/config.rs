use std::env;

pub struct AppConfig {
    pub google_api_key: String,
    pub port: u16,
}

impl AppConfig {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        Self {
            google_api_key: env::var("GOOGLE_API_KEY").expect("GOOGLE_API_KEY required"),
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .unwrap(),
        }
    }
}
