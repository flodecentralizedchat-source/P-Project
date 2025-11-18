use crate::ratelimit::RateLimiter;
use p_project_core::database::MySqlDatabase;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<MySqlDatabase>,
    pub rate_limiter: Arc<RateLimiter>,
    pub strict_rate_limiter: Arc<RateLimiter>,
    pub uvp_engine: Arc<tokio::sync::RwLock<p_project_core::UvpEngine>>,
    pub partner_registry: Arc<tokio::sync::RwLock<p_project_core::PartnerRegistry>>,
    pub ecosystem_graph: Arc<tokio::sync::RwLock<p_project_core::EcosystemGraph>>,
}
