use axum::{
    body::Body,
    http::{self, Request, StatusCode},
    routing::get,
    Router,
};
use tower::ServiceExt;

async fn ok_handler() -> &'static str {
    "ok"
}

#[tokio::test]
async fn test_security_headers_present() {
    let app = Router::new()
        .route("/", get(ok_handler))
        .layer(axum::middleware::from_fn(p_project_api::middleware::security_headers));

    let req = Request::builder()
        .method(http::Method::GET)
        .uri("/")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let headers = resp.headers();
    assert!(headers.get("x-content-type-options").is_some());
    assert!(headers.get("x-frame-options").is_some());
    assert!(headers.get("referrer-policy").is_some());
    assert!(headers.get("strict-transport-security").is_some());
    assert!(headers.get("content-security-policy").is_some());
}

#[tokio::test]
async fn test_admin_ip_allowlist() {
    std::env::set_var("ADMIN_IP_ALLOWLIST", "1.2.3.4");

    let app = Router::new()
        .route("/admin", get(ok_handler))
        .route_layer(axum::middleware::from_fn(p_project_api::middleware::require_admin_ip));

    // Disallowed IP
    let req_bad = Request::builder()
        .method(http::Method::GET)
        .uri("/admin")
        .header("x-forwarded-for", "5.6.7.8")
        .body(Body::empty())
        .unwrap();
    let resp_bad = app.clone().oneshot(req_bad).await.unwrap();
    assert_eq!(resp_bad.status(), StatusCode::FORBIDDEN);

    // Allowed IP
    let req_ok = Request::builder()
        .method(http::Method::GET)
        .uri("/admin")
        .header("x-forwarded-for", "1.2.3.4")
        .body(Body::empty())
        .unwrap();
    let resp_ok = app.oneshot(req_ok).await.unwrap();
    assert_eq!(resp_ok.status(), StatusCode::OK);
}

