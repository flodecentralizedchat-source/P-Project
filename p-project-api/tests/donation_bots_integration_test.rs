//! Integration tests for donation platform webhooks and tip bots
use axum::{
    body::Body,
    http::{self, Request, StatusCode},
    Router,
};
use p_project_api::{handlers, shared::AppState};
use p_project_core::database::MySqlDatabase;
use std::sync::Arc;
use tower::ServiceExt; // for `oneshot`

// Minimal mock DB to satisfy AppState
struct MockDatabase;

#[async_trait::async_trait]
impl MySqlDatabase for MockDatabase {
    async fn new(_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(MockDatabase)
    }
    async fn init_tables(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

#[tokio::test]
async fn test_telegram_webhook_ok() {
    let app_state = AppState {
        db: Arc::new(MockDatabase),
        rate_limiter: Arc::new(p_project_api::ratelimit::RateLimiter::new(100, 60)),
        strict_rate_limiter: Arc::new(p_project_api::ratelimit::RateLimiter::new(10, 60)),
    };

    let app = Router::new()
        .route(
            "/web2/telegram/webhook",
            axum::routing::post(handlers::telegram_webhook),
        )
        .with_state(app_state);

    let request = Request::builder()
        .method(http::Method::POST)
        .uri("/web2/telegram/webhook")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            r#"{ "update_id": 1, "message": {"text": "hi"} }"#,
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_discord_webhook_ok() {
    let app_state = AppState {
        db: Arc::new(MockDatabase),
        rate_limiter: Arc::new(p_project_api::ratelimit::RateLimiter::new(100, 60)),
        strict_rate_limiter: Arc::new(p_project_api::ratelimit::RateLimiter::new(10, 60)),
    };

    let app = Router::new()
        .route(
            "/web2/discord/webhook",
            axum::routing::post(handlers::discord_webhook),
        )
        .with_state(app_state);

    let request = Request::builder()
        .method(http::Method::POST)
        .uri("/web2/discord/webhook")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{ "payload": {"event": "MESSAGE_CREATE"} }"#))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
