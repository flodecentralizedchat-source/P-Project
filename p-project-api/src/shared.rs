use p_project_core::database::MySqlDatabase;
use crate::ratelimit::RateLimiter;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<MySqlDatabase>,
    pub rate_limiter: Arc<RateLimiter>,
    pub strict_rate_limiter: Arc<RateLimiter>,
}
