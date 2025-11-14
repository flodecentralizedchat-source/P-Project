use std::collections::HashMap;
use std::time::Instant;
use tokio::sync::Mutex;

pub struct RateLimiter {
    inner: Mutex<HashMap<String, (u64, Instant)>>,
    window_secs: u64,
    max: u64,
}

impl RateLimiter {
    pub fn from_env() -> Self {
        let window_secs = std::env::var("RATE_LIMIT_WINDOW_SECS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(60);
        let max = std::env::var("RATE_LIMIT_MAX")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(120);
        Self { inner: Mutex::new(HashMap::new()), window_secs, max }
    }

    pub fn from_env_with_prefix(prefix: &str) -> Self {
        let ws_key = format!("{}WINDOW_SECS", prefix);
        let max_key = format!("{}MAX", prefix);
        let window_secs = std::env::var(ws_key)
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(60);
        let max = std::env::var(max_key)
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(20);
        Self { inner: Mutex::new(HashMap::new()), window_secs, max }
    }

    pub fn from_pair(window_secs: u64, max: u64) -> Self {
        Self { inner: Mutex::new(HashMap::new()), window_secs, max }
    }

    pub async fn allow(&self, key: &str) -> bool {
        let mut map = self.inner.lock().await;
        let now = Instant::now();
        let entry = map.entry(key.to_string()).or_insert((0, now));
        let elapsed = now.saturating_duration_since(entry.1).as_secs();
        if elapsed >= self.window_secs {
            entry.0 = 0;
            entry.1 = now;
        }
        if entry.0 >= self.max {
            return false;
        }
        entry.0 += 1;
        true
    }
}
