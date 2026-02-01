use axum::{
    Router,
    body::Body,
    extract::Json,
    http::{Request, StatusCode},
    routing::post,
};
use serde_json::json;
use tower::ServiceExt; // for `oneshot`

// Include the actual DTO from the source to ensure we are testing the real implementation
#[path = "../src/dtos/agent.rs"]
mod agent_dto;
use agent_dto::AgentRequest;

#[test]
fn test_agent_request_deserialization_alias() {
    // Test case 1: POST /agent/run with "message" (alias check)
    let json = r#"{"message": "Hello from alias"}"#;
    let req: AgentRequest = serde_json::from_str(json).expect("Should support 'message' alias");
    assert_eq!(req.query, "Hello from alias");
}

#[test]
fn test_agent_request_deserialization_full() {
    // Test case 2: POST /agent/run with "query", "image", "custom_instruction"
    let json = r#"{
        "query": "What is in this image?",
        "image": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==",
        "custom_instruction": "Be extremely brief",
        "stream": true,
        "session_id": "test-session"
    }"#;
    let req: AgentRequest = serde_json::from_str(json).expect("Should support full payload");
    assert_eq!(req.query, "What is in this image?");
    assert_eq!(req.image, Some("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==".to_string()));
    assert_eq!(
        req.custom_instruction,
        Some("Be extremely brief".to_string())
    );
    assert!(req.stream);
    assert_eq!(req.session_id, Some("test-session".to_string()));
}

#[test]
fn test_agent_request_deserialization_defaults() {
    // Test optional fields and defaults
    let json = r#"{"query": "Hello"}"#;
    let req: AgentRequest = serde_json::from_str(json).expect("Should support minimal payload");
    assert_eq!(req.query, "Hello");
    assert!(!req.stream); // Default value from #[serde(default)]
    assert_eq!(req.session_id, None);
    assert_eq!(req.image, None);
    assert_eq!(req.custom_instruction, None);
}

#[tokio::test]
async fn test_agent_run_endpoint_mock() {
    // This test verifies that the axum router correctly deserializes the AgentRequest
    // with the "message" alias in a simulated HTTP request.

    // Create a mock handler that just returns OK if deserialization succeeds
    let app: Router = Router::new().route(
        "/agent/run",
        post(|Json(req): Json<AgentRequest>| async move {
            if req.query == "hello" {
                StatusCode::OK
            } else {
                StatusCode::BAD_REQUEST
            }
        }),
    );

    // Send a request with "message" instead of "query"
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/agent/run")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({"message": "hello"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_agent_run_endpoint_full_payload() {
    let app: Router = Router::new().route(
        "/agent/run",
        post(|Json(req): Json<AgentRequest>| async move {
            if req.query == "test" && req.custom_instruction == Some("brief".to_string()) {
                StatusCode::OK
            } else {
                StatusCode::BAD_REQUEST
            }
        }),
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/agent/run")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "query": "test",
                        "custom_instruction": "brief",
                        "stream": false
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
