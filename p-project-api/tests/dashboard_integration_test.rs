use axum::{
    body::Body,
    http::{self, Request, StatusCode},
    Router,
};
use p_project_api::{handlers, shared::AppState};
use p_project_core::database::MySqlDatabase;
use std::sync::Arc;
use tower::ServiceExt; // for `oneshot` and `ready`

#[tokio::test]
async fn test_dao_endpoints() {
    // Skip this test if we don't have a database connection
    let db_url = match std::env::var("TEST_DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            println!("Skipping DAO integration test - no TEST_DATABASE_URL");
            return;
        }
    };

    // Initialize database
    let db = MySqlDatabase::new(&db_url).await.expect("Failed to connect to database");
    // Skip table initialization for this test as we're only testing routes

    let app_state = AppState { db: Arc::new(db) };
    let app = Router::new()
        // DAO endpoints
        .route("/dao/proposals", axum::routing::post(handlers::create_proposal))
        .route("/dao/proposals/vote", axum::routing::post(handlers::vote_on_proposal))
        .route("/dao/proposals/tally", axum::routing::post(handlers::tally_votes))
        .route("/dao/proposals/execute", axum::routing::post(handlers::execute_proposal))
        .route("/dao/delegate", axum::routing::post(handlers::delegate_vote))
        .route("/dao/proposals", axum::routing::get(handlers::get_proposals))
        .with_state(app_state);

    // Test create proposal endpoint
    let response = app
        .clone()
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
        .clone()
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
    // Skip this test if we don't have a database connection
    let db_url = match std::env::var("TEST_DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            println!("Skipping staking integration test - no TEST_DATABASE_URL");
            return;
        }
    };

    // Initialize database
    let db = MySqlDatabase::new(&db_url).await.expect("Failed to connect to database");
    // Skip table initialization for this test as we're only testing routes

    let app_state = AppState { db: Arc::new(db) };
    let app = Router::new()
        // Staking endpoints
        .route("/staking/yield", axum::routing::post(handlers::calculate_staking_yield))
        .route("/staking/tiers", axum::routing::get(handlers::get_staking_tiers))
        .with_state(app_state);

    // Test calculate staking yield endpoint
    let response = app
        .clone()
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
        .clone()
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
    // Skip this test if we don't have a database connection
    let db_url = match std::env::var("TEST_DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            println!("Skipping airdrop integration test - no TEST_DATABASE_URL");
            return;
        }
    };

    // Initialize database
    let db = MySqlDatabase::new(&db_url).await.expect("Failed to connect to database");
    // Skip table initialization for this test as we're only testing routes

    let app_state = AppState { db: Arc::new(db) };
    let app = Router::new()
        // Airdrop endpoints
        .route("/airdrop/status", axum::routing::get(handlers::get_airdrop_status))
        .route("/airdrop/recipients", axum::routing::get(handlers::get_airdrop_recipients))
        .with_state(app_state);

    // Test get airdrop status endpoint
    let response = app
        .clone()
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