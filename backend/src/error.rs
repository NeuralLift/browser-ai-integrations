use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Debug)]
pub enum AgentError {
    LlmError(String),
    InvalidRequest(String),
}

impl IntoResponse for AgentError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AgentError::LlmError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AgentError::InvalidRequest(msg) => (StatusCode::BAD_REQUEST, msg),
        };
        (status, message).into_response()
    }
}
