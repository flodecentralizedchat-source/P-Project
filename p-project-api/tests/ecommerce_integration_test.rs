//! Integration tests for e-commerce payment endpoints
use axum::{
    body::Body,
    http::{self, Request, StatusCode},
    Router,
};
use p_project_api::{handlers, ratelimit::RateLimiter, shared::AppState};
use p_project_core::{
    database::MySqlDatabase,
    EcosystemGraph,
    PartnerRegistry,
    UvpEngine,
};
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::ServiceExt; // for `call`, `oneshot`, and `ready`

async fn build_test_app_state() -> AppState {
    let db_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "mysql://root:rootpassword@localhost:3306/p_project".to_string());
    let db = MySqlDatabase::new(&db_url)
        .await
        .expect("failed to build MySQL database");

    let uvp_engine = UvpEngine::new(Vec::new());
    let partner_registry = PartnerRegistry::new();
    let ecosystem_graph = EcosystemGraph::new();

    AppState {
        db: Arc::new(db),
        rate_limiter: Arc::new(RateLimiter::from_pair(100, 60)),
        strict_rate_limiter: Arc::new(RateLimiter::from_pair(10, 60)),
        uvp_engine: Arc::new(RwLock::new(uvp_engine)),
        partner_registry: Arc::new(RwLock::new(partner_registry)),
        ecosystem_graph: Arc::new(RwLock::new(ecosystem_graph)),
    }
}

#[tokio::test]
async fn test_create_ecommerce_integration() {
    let app_state = build_test_app_state().await;

    // Build our application with a route
    let app = Router::new()
        .route(
            "/web2/create-ecommerce-integration",
            axum::routing::post(handlers::create_ecommerce_integration),
        )
        .with_state(app_state.clone());

    // Create a test request
    let request = Request::builder()
        .method(http::Method::POST)
        .uri("/web2/create-ecommerce-integration")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            r#"{
                "config": {
                    "platform": "shopify",
                    "api_key": "test_key",
                    "api_secret": "test_secret",
                    "store_url": "https://test-shop.myshopify.com",
                    "webhook_secret": "webhook_secret"
                }
            }"#,
        ))
        .unwrap();

    // Call the handler
    let response = app.oneshot(request).await.unwrap();

    // Assert the response
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED); // Should fail without JWT
}

#[tokio::test]
async fn test_process_ecommerce_payment() {
    let app_state = build_test_app_state().await;

    // Build our application with a route
    let app = Router::new()
        .route(
            "/web2/process-ecommerce-payment",
            axum::routing::post(handlers::process_ecommerce_payment),
        )
        .with_state(app_state.clone());

    // Create a test request
    let request = Request::builder()
        .method(http::Method::POST)
        .uri("/web2/process-ecommerce-payment")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            r#"{
                "payment_data": {
                    "order_id": "order_123",
                    "customer_wallet": "0x123456789",
                    "amount": 25.99,
                    "currency": "P",
                    "platform": "shopify",
                    "webhook_data": null
                }
            }"#,
        ))
        .unwrap();

    // Call the handler
    let response = app.oneshot(request).await.unwrap();

    // Assert the response
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED); // Should fail without JWT
}

#[tokio::test]
async fn test_verify_webhook_signature() {
    let app_state = build_test_app_state().await;

    // Build our application with a route
    let app = Router::new()
        .route(
            "/web2/verify-webhook",
            axum::routing::post(handlers::verify_webhook_signature),
        )
        .with_state(app_state.clone());

    // Create a test request
    let request = Request::builder()
        .method(http::Method::POST)
        .uri("/web2/verify-webhook")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            r#"{
                "platform": "shopify",
                "body": "test body",
                "signature": "invalid_signature",
                "secret": "test_secret"
            }"#,
        ))
        .unwrap();

    // Call the handler
    let response = app.oneshot(request).await.unwrap();

    // Assert the response
    assert_eq!(response.status(), StatusCode::OK);

    // Get the response body
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Assert the response body
    assert_eq!(body["valid"], false);
}

#[tokio::test]
async fn test_add_digital_goods_product() {
    let app_state = build_test_app_state().await;

    // Build our application with a route
    let app = Router::new()
        .route(
            "/merchant/digital-goods",
            axum::routing::post(handlers::add_digital_goods_product),
        )
        .with_state(app_state.clone());

    // Create a test request
    let request = Request::builder()
        .method(http::Method::POST)
        .uri("/merchant/digital-goods")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            r#"{
                "merchant_id": "merchant_123",
                "product": {
                    "product_id": "product_123",
                    "name": "Test E-book",
                    "description": "A test e-book for digital goods",
                    "price": 9.99,
                    "currency": "P",
                    "category": "ebook",
                    "country_restrictions": ["CN", "RU"],
                    "language_availability": ["en", "es"],
                    "digital_delivery_method": "download",
                    "download_url": "https://example.com/download/test-ebook.pdf",
                    "license_key_template": null,
                    "created_at": "2023-01-01T00:00:00",
                    "updated_at": "2023-01-01T00:00:00"
                }
            }"#,
        ))
        .unwrap();

    // Call the handler
    let response = app.oneshot(request).await.unwrap();

    // Assert the response
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED); // Should fail without JWT
}

#[tokio::test]
async fn test_purchase_digital_goods() {
    let app_state = build_test_app_state().await;

    // Build our application with a route
    let app = Router::new()
        .route(
            "/merchant/digital-goods/purchase",
            axum::routing::post(handlers::purchase_digital_goods),
        )
        .with_state(app_state.clone());

    // Create a test request
    let request = Request::builder()
        .method(http::Method::POST)
        .uri("/merchant/digital-goods/purchase")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            r#"{
                "product_id": "product_123",
                "customer_id": "customer_123",
                "customer_country": "US",
                "customer_language": "en"
            }"#,
        ))
        .unwrap();

    // Call the handler
    let response = app.oneshot(request).await.unwrap();

    // Assert the response
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED); // Should fail without JWT
}
