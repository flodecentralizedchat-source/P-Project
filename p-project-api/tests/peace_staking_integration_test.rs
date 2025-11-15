use axum::{
    body::Body,
    http::{self, Request, StatusCode},
    routing::post,
    Router,
};
use serde_json::json;
use tower::ServiceExt; // for `call`, `oneshot`, and `ready`

// Import the handlers we want to test
use p_project_api::handlers::{
    calculate_peace_staking_bonus, record_donation_event, CalculatePeaceStakingBonusRequest,
    RecordDonationEventRequest,
};

// Mock middleware claims for testing
mod mock_middleware {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Claims {
        pub sub: String,
        pub role: Option<String>,
        pub exp: usize,
    }

    impl Claims {
        pub fn new(sub: &str, role: Option<&str>) -> Self {
            Self {
                sub: sub.to_string(),
                role: role.map(|s| s.to_string()),
                exp: 10000000000, // Far in the future
            }
        }
    }
}

#[tokio::test]
async fn test_record_donation_event_success() {
    // Create a mock app with just the endpoint we want to test
    let app = Router::new().route(
        "/peace-staking/record-donation",
        post(record_donation_event),
    );

    // Create a mock JWT claims extension
    let claims = mock_middleware::Claims::new("test_user", Some("user"));

    // Prepare the request body
    let request_body = RecordDonationEventRequest {
        event_id: "event_123".to_string(),
        user_id: "user_456".to_string(),
        donation_amount: 100.0,
    };

    // Make the request
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/peace-staking/record-donation")
                .header(http::header::CONTENT_TYPE, "application/json")
                .extension(claims)
                .body(Body::from(serde_json::to_vec(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Check the response status
    assert_eq!(response.status(), StatusCode::OK);

    // Parse the response body
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let response_body: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Check that the response indicates success
    assert_eq!(response_body["success"], true);
    assert_eq!(
        response_body["message"],
        "Donation event recorded successfully"
    );
}

#[tokio::test]
async fn test_calculate_peace_staking_bonus_success() {
    // Create a mock app with just the endpoint we want to test
    let app = Router::new().route(
        "/peace-staking/calculate-bonus",
        post(calculate_peace_staking_bonus),
    );

    // Create a mock JWT claims extension
    let claims = mock_middleware::Claims::new("test_user", Some("user"));

    // Prepare the request body
    let request_body = CalculatePeaceStakingBonusRequest {
        user_id: "user_456".to_string(),
    };

    // Make the request
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/peace-staking/calculate-bonus")
                .header(http::header::CONTENT_TYPE, "application/json")
                .extension(claims)
                .body(Body::from(serde_json::to_vec(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Check the response status
    assert_eq!(response.status(), StatusCode::OK);

    // Parse the response body
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let response_body: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Check that the response indicates success
    assert_eq!(
        response_body["message"],
        "Peace staking bonus calculated successfully"
    );
}

#[tokio::test]
async fn test_record_donation_event_missing_fields() {
    // Create a mock app with just the endpoint we want to test
    let app = Router::new().route(
        "/peace-staking/record-donation",
        post(record_donation_event),
    );

    // Create a mock JWT claims extension
    let claims = mock_middleware::Claims::new("test_user", Some("user"));

    // Prepare an incomplete request body (missing donation_amount)
    let request_body = json!({
        "event_id": "event_123",
        "user_id": "user_456"
        // Missing donation_amount
    });

    // Make the request
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/peace-staking/record-donation")
                .header(http::header::CONTENT_TYPE, "application/json")
                .extension(claims)
                .body(Body::from(serde_json::to_vec(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Check that we get a bad request response due to missing fields
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_calculate_peace_staking_bonus_missing_fields() {
    // Create a mock app with just the endpoint we want to test
    let app = Router::new().route(
        "/peace-staking/calculate-bonus",
        post(calculate_peace_staking_bonus),
    );

    // Create a mock JWT claims extension
    let claims = mock_middleware::Claims::new("test_user", Some("user"));

    // Prepare an incomplete request body (missing user_id)
    let request_body = json!({
        // Missing user_id
    });

    // Make the request
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/peace-staking/calculate-bonus")
                .header(http::header::CONTENT_TYPE, "application/json")
                .extension(claims)
                .body(Body::from(serde_json::to_vec(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Check that we get a bad request response due to missing fields
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
