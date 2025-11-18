// Middleware and security utilities for the P-Project API

use axum::http::Method;
use axum::{
    body::Body,
    http::{header, HeaderValue, Request, StatusCode},
    middleware::Next,
    response::Response,
};
use http::HeaderName;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;
use tower_http::cors::{Any, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use axum::http::HeaderMap;

pub fn init_middleware() {
    println!("Middleware initialized");
}

// Build a CORS layer driven by environment variables.
// CORS_ALLOWED_ORIGINS: comma-separated list of origins, or "*" to allow all
// CORS_ALLOW_CREDENTIALS: "true" to allow credentials (default false)
pub fn build_cors_from_env() -> CorsLayer {
    let allow_credentials = env::var("CORS_ALLOW_CREDENTIALS")
        .ok()
        .filter(|v| v.eq_ignore_ascii_case("true"))
        .is_some();

    let mut layer = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::OPTIONS])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE, header::ACCEPT]);

    if let Ok(origins) = env::var("CORS_ALLOWED_ORIGINS") {
        if origins.trim() == "*" {
            layer = layer.allow_origin(Any);
        } else {
            let list: Vec<HeaderValue> = origins
                .split(',')
                .filter_map(|s| HeaderValue::from_str(s.trim()).ok())
                .collect();
            if !list.is_empty() {
                layer = layer.allow_origin(list);
            }
        }
    } else {
        // Default to no origin restrictions in dev to ease local testing
        layer = layer.allow_origin(Any);
    }

    if allow_credentials {
        layer = layer.allow_credentials(true);
    }

    layer
}

// Build a request body size limit layer (bytes) from env, default 1 MB
pub fn build_body_limit_from_env() -> RequestBodyLimitLayer {
    let max_bytes = std::env::var("MAX_REQUEST_BYTES")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(1_048_576);
    RequestBodyLimitLayer::new(max_bytes)
}

// Simple global rate limit (requests per second) using tower's RateLimitLayer
// RATE_LIMIT_RPS: number per second (default 10)
pub fn build_rate_limit_from_env() -> tower_http::limit::rate::RateLimitLayer {
    let max = std::env::var("RATE_LIMIT_MAX")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(120);
    let window = std::env::var("RATE_LIMIT_WINDOW_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(60);
    tower_http::limit::rate::RateLimitLayer::new(max, Duration::from_secs(window))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    #[allow(dead_code)]
    pub iat: Option<usize>,
    #[allow(dead_code)]
    pub role: Option<String>,
}

fn unauthorized_response() -> Response {
    let body = serde_json::json!({"error": "unauthorized"}).to_string();
    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body))
        .unwrap()
}

// Simple JWT verification middleware (HS256) using JWT_SECRET env var
pub async fn require_jwt(mut req: Request, next: Next) -> Response {
    let secret = match env::var("JWT_SECRET") {
        Ok(v) => v,
        Err(_) => return unauthorized_response(),
    };

    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    // Expect format: "Bearer <token>"
    let token = auth_header
        .strip_prefix("Bearer ")
        .map(|s| s.trim())
        .unwrap_or("");

    if token.is_empty() {
        return unauthorized_response();
    }

    let validation = Validation::new(Algorithm::HS256);
    let key = DecodingKey::from_secret(secret.as_bytes());
    match decode::<Claims>(token, &key, &validation) {
        Ok(data) => {
            req.extensions_mut().insert(data.claims);
        }
        Err(_) => return unauthorized_response(),
    }

    next.run(req).await
}

// Add common security headers to all responses
pub async fn security_headers(mut req: Request, next: Next) -> Response {
    let mut res = next.run(req).await;
    let headers = res.headers_mut();
    headers.insert(
        HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    );
    headers.insert(
        HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("no-referrer"),
    );
    headers.insert(
        HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static(
            "geolocation=(), microphone=(), camera=(), payment=()",
        ),
    );
    headers.insert(
        HeaderName::from_static("strict-transport-security"),
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );
    // Basic default CSP safe for APIs (disallows inline/script eval by default)
    headers.insert(
        HeaderName::from_static("content-security-policy"),
        HeaderValue::from_static("default-src 'none'"),
    );
    res
}

// Admin IP allowlist middleware (checks X-Forwarded-For first)
pub async fn require_admin_ip(req: Request, next: Next) -> Response {
    let allowlist = std::env::var("ADMIN_IP_ALLOWLIST")
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();

    // If no allowlist configured, allow all (opt-in)
    if allowlist.is_empty() {
        return next.run(req).await;
    }

    let ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| req.extensions().get::<std::net::SocketAddr>().map(|sa| sa.ip().to_string()))
        .unwrap_or_else(|| "unknown".to_string());

    if allowlist.iter().any(|a| a == &ip) {
        next.run(req).await
    } else {
        let body = serde_json::json!({"error": "forbidden_ip"}).to_string();
        Response::builder()
            .status(StatusCode::FORBIDDEN)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body))
            .unwrap()
    }
}

// Per-IP rate limit using AppState's limiter (skips OPTIONS)
pub async fn require_rate_limit(req: Request, next: Next) -> Response {
    use axum::http::Method;
    if req.method() == Method::OPTIONS {
        return next.run(req).await;
    }
    let ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string());

    let path = req.uri().path().to_string();
    let method = req.method().clone();
    let is_strict = method == Method::POST
        && matches!(
            path.as_str(),
            "/airdrop/create"
                | "/dao/proposals/execute"
                | "/web2/create-donation-widget"
                | "/web2/create-youtube-tip-config"
                | "/web2/register-messaging-bot"
                | "/web2/telegram/webhook"
                | "/web2/discord/webhook"
        );

    if let Some(state) = req.extensions().get::<crate::shared::AppState>() {
        let limiter = if is_strict {
            &state.strict_rate_limiter
        } else {
            &state.rate_limiter
        };
        let key = format!("{}:{}", ip, if is_strict { path } else { "*".to_string() });
        if !limiter.allow(&key).await {
            let body = serde_json::json!({"error": "rate_limited"}).to_string();
            return Response::builder()
                .status(StatusCode::TOO_MANY_REQUESTS)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .unwrap();
        }
    }

    next.run(req).await
}

// Admin-only guard using JWT role claim
pub async fn require_admin(req: Request, next: Next) -> Response {
    if let Some(claims) = req.extensions().get::<Claims>() {
        if matches!(claims.role.as_deref(), Some("admin")) {
            return next.run(req).await;
        }
    }
    let body = serde_json::json!({"error": "forbidden"}).to_string();
    Response::builder()
        .status(StatusCode::FORBIDDEN)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body))
        .unwrap()
}
