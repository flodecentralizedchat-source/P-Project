use axum::{
    body::Body,
    http::{self, Request, StatusCode},
    Router,
};
use tower::ServiceExt;

#[tokio::test]
async fn test_get_exchange_listings_strategy_public() {
    // Build a minimal app exposing only the new public route
    let app = Router::new().route(
        "/strategy/exchange-listings",
        axum::routing::get(p_project_api::handlers::get_exchange_listings_strategy),
    );

    let req = Request::builder()
        .method(http::Method::GET)
        .uri("/strategy/exchange-listings")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(resp.into_body()).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Basic shape assertions
    assert!(v.get("strategy").is_some());
    assert!(v["strategy"].get("dex_strategy").is_some());
    assert_eq!(
        v["strategy"]["dex_strategy"]["start_with_permissionless"],
        true
    );
}
