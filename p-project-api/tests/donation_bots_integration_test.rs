//! Integration tests for donation platform webhooks and tip bots
use axum::{
    body::Body,
    http::{self, Request, StatusCode},
    Router,
};
use p_project_api::handlers;
use tower::ServiceExt; // for `oneshot`

#[tokio::test]
async fn test_telegram_webhook_ok() {
    std::env::set_var("WEBHOOK_TOKEN", "testtoken");

    let app = Router::new()
        .route(
            "/web2/telegram/webhook",
            axum::routing::post(handlers::telegram_webhook),
        );

    let request = Request::builder()
        .method(http::Method::POST)
        .uri("/web2/telegram/webhook")
        .header(http::header::CONTENT_TYPE, "application/json")
        .header("x-webhook-token", "testtoken")
        .body(Body::from(
            r#"{ "update_id": 1, "message": {"text": "hi"} }"#,
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_discord_webhook_ok() {
    std::env::set_var("WEBHOOK_TOKEN", "testtoken");

    let app = Router::new()
        .route(
            "/web2/discord/webhook",
            axum::routing::post(handlers::discord_webhook),
        );

    let request = Request::builder()
        .method(http::Method::POST)
        .uri("/web2/discord/webhook")
        .header(http::header::CONTENT_TYPE, "application/json")
        .header("x-webhook-token", "testtoken")
        .body(Body::from(r#"{ "payload": {"event": "MESSAGE_CREATE"} }"#))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
