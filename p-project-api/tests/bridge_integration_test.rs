use axum::{
    body::Body,
    http::{self, Request, StatusCode},
    Router,
};
use p_project_api::{handlers, shared::AppState};
use p_project_core::database::MySqlDatabase;
use serde_json::json;
use std::sync::Arc;
use tower::ServiceExt;

/// Test bridge endpoints
#[tokio::test]
async fn test_bridge_endpoints() -> Result<(), Box<dyn std::error::Error>> {
    // Skip this test if we don't have a database connection
    let db_url = match std::env::var("TEST_DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            println!("Skipping bridge integration test - no TEST_DATABASE_URL");
            return Ok(());
        }
    };

    // Initialize database
    let db = MySqlDatabase::new(&db_url).await?;
    db.init_tables().await?;

    let app_state = AppState { db: Arc::new(db) };
    let app = Router::new()
        .route("/bridge", axum::routing::post(handlers::bridge_tokens))
        .route(
            "/bridge/status",
            axum::routing::post(handlers::get_bridge_status),
        )
        .route(
            "/metrics",
            axum::routing::get(handlers::get_performance_metrics),
        )
        .with_state(app_state);

    // Test bridge endpoint with invalid amount
    let request = Request::builder()
        .method(http::Method::POST)
        .uri("/bridge")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "user_id": "user1",
                "from_chain": "Ethereum",
                "to_chain": "Solana",
                "amount": -100.0
            })
            .to_string(),
        ))?;

    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Test bridge status endpoint with invalid transaction ID
    let request = Request::builder()
        .method(http::Method::POST)
        .uri("/bridge/status")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "transaction_id": "invalid-tx-id"
            })
            .to_string(),
        ))?;

    let response = app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // Test metrics endpoint
    let request = Request::builder()
        .method(http::Method::GET)
        .uri("/metrics")
        .body(Body::empty())?;

    let response = app.oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::OK);

    Ok(())
}
