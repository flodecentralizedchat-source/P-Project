use axum::{
    body::Body,
    http::{self, Request, StatusCode},
    Router,
};
use p_project_api::handlers;
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn staking_yield_accepts_decimal_numbers() {
    let app = Router::new()
        .route("/staking/yield", axum::routing::post(handlers::calculate_staking_yield));

    // JSON numbers should deserialize into Decimal
    let req = Request::builder()
        .method(http::Method::POST)
        .uri("/staking/yield")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({
            "amount": 1234.56789012,
            "duration_days": 100
        }).to_string()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Negative amounts should be rejected
    let neg = Request::builder()
        .method(http::Method::POST)
        .uri("/staking/yield")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({
            "amount": -1,
            "duration_days": 30
        }).to_string()))
        .unwrap();
    let res = app.oneshot(neg).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

