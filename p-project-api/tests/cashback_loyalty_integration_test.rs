//! Integration tests for cashback and loyalty endpoints
use axum::{
    body::Body,
    http::{self, Request, StatusCode},
    Router,
};
use p_project_api::{handlers, shared::AppState};
use p_project_core::database::MySqlDatabase;
use std::sync::Arc;
use tower::ServiceExt; // for `call`, `oneshot`, and `ready`

// Mock database for testing
struct MockDatabase;

#[async_trait::async_trait]
impl MySqlDatabase for MockDatabase {
    // Implement minimal required methods for testing
    async fn new(_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(MockDatabase)
    }

    async fn init_tables(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    // Add other required methods as needed
}

#[tokio::test]
async fn test_configure_cashback() {
    // Create app state with mock database
    let app_state = AppState {
        db: Arc::new(MockDatabase),
        rate_limiter: Arc::new(p_project_api::ratelimit::RateLimiter::new(100, 60)),
        strict_rate_limiter: Arc::new(p_project_api::ratelimit::RateLimiter::new(10, 60)),
    };

    // Build our application with a route
    let app = Router::new()
        .route(
            "/merchant/cashback/configure",
            axum::routing::post(handlers::configure_cashback),
        )
        .with_state(app_state);

    // Create a test request
    let request = Request::builder()
        .method(http::Method::POST)
        .uri("/merchant/cashback/configure")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            r#"{
                "merchant_id": "merchant_123",
                "cashback_percentage": 5.0,
                "min_purchase_amount": 10.0,
                "max_cashback_amount": 50.0,
                "loyalty_points_per_coin": 1.0
            }"#,
        ))
        .unwrap();

    // Call the handler
    let response = app.oneshot(request).await.unwrap();

    // Assert the response
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED); // Should fail without JWT
}

#[tokio::test]
async fn test_process_purchase_with_cashback() {
    // Create app state with mock database
    let app_state = AppState {
        db: Arc::new(MockDatabase),
        rate_limiter: Arc::new(p_project_api::ratelimit::RateLimiter::new(100, 60)),
        strict_rate_limiter: Arc::new(p_project_api::ratelimit::RateLimiter::new(10, 60)),
    };

    // Build our application with a route
    let app = Router::new()
        .route(
            "/merchant/cashback/process-purchase",
            axum::routing::post(handlers::process_purchase_with_cashback),
        )
        .with_state(app_state);

    // Create a test request
    let request = Request::builder()
        .method(http::Method::POST)
        .uri("/merchant/cashback/process-purchase")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            r#"{
                "merchant_id": "merchant_123",
                "customer_id": "customer_123",
                "purchase_amount": 100.0,
                "customer_wallet": "0x123456789"
            }"#,
        ))
        .unwrap();

    // Call the handler
    let response = app.oneshot(request).await.unwrap();

    // Assert the response
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED); // Should fail without JWT
}

#[tokio::test]
async fn test_get_customer_loyalty_points() {
    // Create app state with mock database
    let app_state = AppState {
        db: Arc::new(MockDatabase),
        rate_limiter: Arc::new(p_project_api::ratelimit::RateLimiter::new(100, 60)),
        strict_rate_limiter: Arc::new(p_project_api::ratelimit::RateLimiter::new(10, 60)),
    };

    // Build our application with a route
    let app = Router::new()
        .route(
            "/merchant/cashback/loyalty-points",
            axum::routing::post(handlers::get_customer_loyalty_points),
        )
        .with_state(app_state);

    // Create a test request
    let request = Request::builder()
        .method(http::Method::POST)
        .uri("/merchant/cashback/loyalty-points")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            r#"{
                "customer_id": "customer_123",
                "merchant_id": "merchant_123"
            }"#,
        ))
        .unwrap();

    // Call the handler
    let response = app.oneshot(request).await.unwrap();

    // Assert the response
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_redeem_loyalty_points() {
    // Create app state with mock database
    let app_state = AppState {
        db: Arc::new(MockDatabase),
        rate_limiter: Arc::new(p_project_api::ratelimit::RateLimiter::new(100, 60)),
        strict_rate_limiter: Arc::new(p_project_api::ratelimit::RateLimiter::new(10, 60)),
    };

    // Build our application with a route
    let app = Router::new()
        .route(
            "/merchant/cashback/redeem-points",
            axum::routing::post(handlers::redeem_loyalty_points),
        )
        .with_state(app_state);

    // Create a test request
    let request = Request::builder()
        .method(http::Method::POST)
        .uri("/merchant/cashback/redeem-points")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            r#"{
                "customer_id": "customer_123",
                "merchant_id": "merchant_123",
                "points_to_redeem": 50.0
            }"#,
        ))
        .unwrap();

    // Call the handler
    let response = app.oneshot(request).await.unwrap();

    // Assert the response
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED); // Should fail without JWT
}

#[tokio::test]
async fn test_get_cashback_transaction() {
    // Create app state with mock database
    let app_state = AppState {
        db: Arc::new(MockDatabase),
        rate_limiter: Arc::new(p_project_api::ratelimit::RateLimiter::new(100, 60)),
        strict_rate_limiter: Arc::new(p_project_api::ratelimit::RateLimiter::new(10, 60)),
    };

    // Build our application with a route
    let app = Router::new()
        .route(
            "/merchant/cashback/transaction",
            axum::routing::post(handlers::get_cashback_transaction),
        )
        .with_state(app_state);

    // Create a test request
    let request = Request::builder()
        .method(http::Method::POST)
        .uri("/merchant/cashback/transaction")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            r#"{
                "transaction_id": "cashback_123"
            }"#,
        ))
        .unwrap();

    // Call the handler
    let response = app.oneshot(request).await.unwrap();

    // Assert the response
    assert_eq!(response.status(), StatusCode::OK);
}
