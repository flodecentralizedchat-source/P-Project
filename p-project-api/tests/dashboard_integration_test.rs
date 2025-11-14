use axum::{
    body::Body,
    http::{self, Request, StatusCode},
    Router,
};
use p_project_api::{handlers, shared::AppState};
use p_project_core::database::MySqlDatabase;
use std::sync::Arc;
use tower::ServiceExt; // for `oneshot` and `ready`

// Helper function to create a test app
fn app() -> Router {
    // In a real test, we would use a mock database
    // For now, we'll create a minimal app for testing routes
    let db = MySqlDatabase::new("mysql://test:test@localhost/test").expect("Failed to create test DB");
    let app_state = AppState { db: Arc::new(db) };
    
    Router::new()
        // DAO endpoints
        .route("/dao/proposals", axum::routing::post(handlers::create_proposal))
        .route("/dao/proposals/vote", axum::routing::post(handlers::vote_on_proposal))
        .route("/dao/proposals/tally", axum::routing::post(handlers::tally_votes))
        .route("/dao/proposals/execute", axum::routing::post(handlers::execute_proposal))
        .route("/dao/delegate", axum::routing::post(handlers::delegate_vote))
        .route("/dao/proposals", axum::routing::get(handlers::get_proposals))
        // Staking endpoints
        .route("/staking/yield", axum::routing::post(handlers::calculate_staking_yield))
        .route("/staking/tiers", axum::routing::get(handlers::get_staking_tiers))
        // Airdrop endpoints
        .route("/airdrop/status", axum::routing::get(handlers::get_airdrop_status))
        .route("/airdrop/recipients", axum::routing::get(handlers::get_airdrop_recipients))
        .with_state(app_state)
}

#[tokio::test]
async fn test_dao_endpoints() {
    let app = app();

    // Test create proposal endpoint
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/dao/proposals")
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"title": "Test Proposal", "description": "A test proposal", "creator_id": "user1"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Test get proposals endpoint
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri("/dao/proposals")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_staking_endpoints() {
    let app = app();

    // Test calculate staking yield endpoint
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/staking/yield")
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"amount": 1000.0, "duration_days": 365}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Test get staking tiers endpoint
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri("/staking/tiers")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_airdrop_endpoints() {
    let app = app();

    // Test get airdrop status endpoint
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri("/airdrop/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Test get airdrop recipients endpoint
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri("/airdrop/recipients")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}