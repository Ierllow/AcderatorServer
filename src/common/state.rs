use super::error::AppError;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::MySqlPool,
    pub rate_limiter: RateLimiter,
}

#[derive(Clone)]
pub struct RateLimiter {
    clients: Arc<Mutex<HashMap<String, RateLimitEntry>>>,
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

struct RateLimitEntry {
    count: u32,
    window_started_at: Instant,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn check(
        &self,
        key: String,
        max_requests: u32,
        window_seconds: u64,
    ) -> Result<(), AppError> {
        if max_requests == 0 {
            return Ok(());
        }

        let now = Instant::now();
        let window = Duration::from_secs(window_seconds);
        let mut clients = self
            .clients
            .lock()
            .map_err(|_| AppError::ServiceFailure("rate limit state unavailable".into()))?;
        clients.retain(|_, entry| now.duration_since(entry.window_started_at) <= window);

        let entry = clients.entry(key).or_insert(RateLimitEntry {
            count: 0,
            window_started_at: now,
        });

        if now.duration_since(entry.window_started_at) > window {
            entry.count = 0;
            entry.window_started_at = now;
        }

        if entry.count >= max_requests {
            return Err(AppError::TooManyRequests);
        }

        entry.count += 1;
        Ok(())
    }
}
