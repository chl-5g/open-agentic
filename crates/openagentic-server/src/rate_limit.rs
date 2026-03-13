//! Simple in-memory rate limiting middleware (token bucket)

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

/// Rate limiter state
#[derive(Clone)]
pub struct RateLimiter {
    /// Per-key buckets: key -> (tokens_remaining, last_refill_time)
    buckets: Arc<Mutex<HashMap<String, Bucket>>>,
    /// Max tokens (burst capacity)
    max_tokens: u32,
    /// Tokens refilled per second
    refill_rate: f64,
    /// Window for refill calculation
    window_secs: f64,
}

struct Bucket {
    tokens: f64,
    last_refill: Instant,
}

impl RateLimiter {
    /// Create a rate limiter.
    /// - `max_per_window`: max requests allowed per window
    /// - `window_secs`: time window in seconds
    pub fn new(max_per_window: u32, window_secs: f64) -> Self {
        Self {
            buckets: Arc::new(Mutex::new(HashMap::new())),
            max_tokens: max_per_window,
            refill_rate: max_per_window as f64 / window_secs,
            window_secs,
        }
    }

    /// Check if a request is allowed for the given key
    pub async fn check(&self, key: &str) -> bool {
        let mut buckets = self.buckets.lock().await;
        let now = Instant::now();

        let bucket = buckets.entry(key.to_string()).or_insert(Bucket {
            tokens: self.max_tokens as f64,
            last_refill: now,
        });

        // Refill tokens based on elapsed time
        let elapsed = now.duration_since(bucket.last_refill).as_secs_f64();
        bucket.tokens = (bucket.tokens + elapsed * self.refill_rate).min(self.max_tokens as f64);
        bucket.last_refill = now;

        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    /// Cleanup old entries (call periodically)
    pub async fn cleanup(&self) {
        let mut buckets = self.buckets.lock().await;
        let now = Instant::now();
        buckets.retain(|_, b| {
            now.duration_since(b.last_refill).as_secs() < 300 // keep for 5 min
        });
    }
}

/// Login rate limit config (shared via Extension)
#[derive(Clone)]
pub struct LoginRateLimiter(pub Arc<RateLimiter>);

/// API rate limit config (shared via Extension)
#[derive(Clone)]
pub struct ApiRateLimiter(pub Arc<RateLimiter>);

/// Rate limit middleware for login endpoint
pub async fn login_rate_limit_middleware(request: Request, next: Next) -> Response {
    let path = request.uri().path().to_string();

    // Only apply to login endpoints
    if path != "/api/auth/login" && path != "/api/auth/token" {
        return next.run(request).await;
    }

    let limiter = request.extensions().get::<LoginRateLimiter>().cloned();
    if let Some(limiter) = limiter {
        // Use client IP as key (from X-Forwarded-For or peer addr)
        let key = extract_client_ip(&request);
        if !limiter.0.check(&key).await {
            tracing::warn!("Login rate limit exceeded for IP: {}", key);
            return (
                StatusCode::TOO_MANY_REQUESTS,
                Json(serde_json::json!({
                    "error": "Too many login attempts, please try again later",
                    "code": 429
                })),
            )
                .into_response();
        }
    }

    next.run(request).await
}

fn extract_client_ip(request: &Request) -> String {
    // Try X-Forwarded-For header first
    if let Some(forwarded) = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
    {
        if let Some(first_ip) = forwarded.split(',').next() {
            return first_ip.trim().to_string();
        }
    }
    // Fallback to a generic key
    "unknown".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_allows_within_limit() {
        let limiter = RateLimiter::new(3, 60.0); // 3 per minute
        assert!(limiter.check("test").await);
        assert!(limiter.check("test").await);
        assert!(limiter.check("test").await);
        assert!(!limiter.check("test").await); // 4th should fail
    }

    #[tokio::test]
    async fn test_rate_limiter_independent_keys() {
        let limiter = RateLimiter::new(1, 60.0);
        assert!(limiter.check("a").await);
        assert!(limiter.check("b").await); // different key
        assert!(!limiter.check("a").await); // a exhausted
    }
}
