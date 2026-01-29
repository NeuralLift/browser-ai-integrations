use serde::Serialize;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
}
