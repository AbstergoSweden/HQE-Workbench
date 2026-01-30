//! Token bucket rate limiter for API calls
//!
//! Provides rate limiting to prevent exceeding API provider limits.
//! Uses a token bucket algorithm that supports:
//! - Requests per minute (RPM) limiting
//! - Tokens per minute (TPM) limiting
//!
//! # Example
//! ```
//! use hqe_openai::rate_limiter::{RateLimiter, RateLimitConfig};
//! use std::time::Duration;
//!
//! let config = RateLimitConfig {
//!     requests_per_minute: 60,
//!     tokens_per_minute: Some(10000),
//! };
//! let limiter = RateLimiter::new(config);
//! ```

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex as TokioMutex;
use tracing::{debug, trace};

/// Configuration for rate limiting
#[derive(Debug, Clone, Copy)]
pub struct RateLimitConfig {
    /// Maximum requests allowed per minute
    pub requests_per_minute: u32,
    /// Maximum tokens allowed per minute (optional)
    pub tokens_per_minute: Option<u32>,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            tokens_per_minute: None,
        }
    }
}

impl RateLimitConfig {
    /// Create a config for OpenAI tier 1 limits (60 RPM, 60k TPM)
    pub fn openai_tier1() -> Self {
        Self {
            requests_per_minute: 60,
            tokens_per_minute: Some(60000),
        }
    }

    /// Create a config for OpenAI tier 2 limits (3000 RPM, 250k TPM)
    pub fn openai_tier2() -> Self {
        Self {
            requests_per_minute: 3000,
            tokens_per_minute: Some(250000),
        }
    }

    /// Create a config for local/development use (no limits)
    pub fn unlimited() -> Self {
        Self {
            requests_per_minute: u32::MAX,
            tokens_per_minute: None,
        }
    }
}

/// Internal state of the token bucket
#[derive(Debug)]
struct TokenBucket {
    /// Current number of tokens available
    tokens: f64,
    /// Maximum tokens the bucket can hold
    max_tokens: f64,
    /// Rate at which tokens are added (tokens per second)
    refill_rate: f64,
    /// Last time the bucket was updated
    last_update: Instant,
}

impl TokenBucket {
    /// Create a new token bucket
    fn new(max_tokens: f64, refill_rate: f64) -> Self {
        Self {
            tokens: max_tokens,
            max_tokens,
            refill_rate,
            last_update: Instant::now(),
        }
    }

    /// Refill tokens based on elapsed time
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update).as_secs_f64();
        let tokens_to_add = elapsed * self.refill_rate;

        self.tokens = (self.tokens + tokens_to_add).min(self.max_tokens);
        self.last_update = now;

        trace!(
            "Token bucket refilled: {} / {} tokens",
            self.tokens,
            self.max_tokens
        );
    }

    /// Try to consume tokens from the bucket
    /// Returns true if successful, false if not enough tokens
    fn try_consume(&mut self, amount: f64) -> bool {
        self.refill();

        if self.tokens >= amount {
            self.tokens -= amount;
            trace!("Consumed {} tokens, {} remaining", amount, self.tokens);
            true
        } else {
            false
        }
    }

    /// Calculate time until enough tokens are available
    fn time_until_available(&self, amount: f64) -> Duration {
        if self.tokens >= amount {
            Duration::ZERO
        } else {
            let tokens_needed = amount - self.tokens;
            let seconds_needed = tokens_needed / self.refill_rate;
            Duration::from_secs_f64(seconds_needed)
        }
    }
}

/// Rate limiter using token bucket algorithm
#[derive(Debug, Clone)]
pub struct RateLimiter {
    /// Token bucket for request rate limiting
    request_bucket: Arc<TokioMutex<TokenBucket>>,
    /// Token bucket for token rate limiting (optional)
    token_bucket: Option<Arc<TokioMutex<TokenBucket>>>,
    /// Configuration
    config: RateLimitConfig,
}

impl RateLimiter {
    /// Create a new rate limiter with the given configuration
    pub fn new(config: RateLimitConfig) -> Self {
        // Convert RPM to tokens per second
        let request_refill_rate = config.requests_per_minute as f64 / 60.0;
        let request_bucket =
            TokenBucket::new(config.requests_per_minute as f64, request_refill_rate);

        // Optional token bucket for TPM limiting
        let token_bucket = config.tokens_per_minute.map(|tpm| {
            let token_refill_rate = tpm as f64 / 60.0;
            TokenBucket::new(tpm as f64, token_refill_rate)
        });

        Self {
            request_bucket: Arc::new(TokioMutex::new(request_bucket)),
            token_bucket: token_bucket.map(|tb| Arc::new(TokioMutex::new(tb))),
            config,
        }
    }

    /// Acquire permission to make a request
    /// Waits if necessary until rate limits allow the request
    ///
    /// # Arguments
    /// * `token_count` - Optional number of tokens this request will consume
    ///   (for TPM limiting)
    ///
    /// # Example
    /// ```rust
    /// # async fn example() {
    /// # use hqe_openai::rate_limiter::{RateLimiter, RateLimitConfig};
    /// let limiter = RateLimiter::new(RateLimitConfig::default());
    ///
    /// // Acquire permission for a request
    /// limiter.acquire(None).await;
    ///
    /// // Make your API call here
    ///
    /// // Or with token count for TPM limiting
    /// limiter.acquire(Some(1000)).await;
    /// # }
    /// ```
    pub async fn acquire(&self, token_count: Option<u32>) {
        let mut request_bucket = self.request_bucket.lock().await;

        loop {
            // Try to consume a request token
            if request_bucket.try_consume(1.0) {
                // If we have token-based limiting and a token count was provided
                if let (Some(bucket), Some(tokens)) = (&self.token_bucket, token_count) {
                    let mut token_bucket = bucket.lock().await;
                    let tokens_f64 = tokens as f64;

                    if token_bucket.try_consume(tokens_f64) {
                        return; // Success!
                    }
                    // Rollback request token if token bucket fails
                    request_bucket.tokens += 1.0;
                } else {
                    return; // Success!
                }
            }

            let wait_time = request_bucket.time_until_available(1.0);
            drop(request_bucket);

            debug!("Rate limit hit, waiting {:?} for request bucket", wait_time);
            tokio::time::sleep(wait_time).await;

            request_bucket = self.request_bucket.lock().await;
        }
    }

    /// Try to acquire permission without waiting
    /// Returns true if successful, false if rate limited
    pub async fn try_acquire(&self, token_count: Option<u32>) -> bool {
        let mut request_bucket = self.request_bucket.lock().await;

        if !request_bucket.try_consume(1.0) {
            return false;
        }

        if let (Some(bucket), Some(tokens)) = (&self.token_bucket, token_count) {
            let mut token_bucket = bucket.lock().await;
            let tokens_f64 = tokens as f64;
            if !token_bucket.try_consume(tokens_f64) {
                // Rollback request token
                request_bucket.tokens += 1.0;
                return false;
            }
        }

        true
    }

    /// Get current configuration
    pub fn config(&self) -> &RateLimitConfig {
        &self.config
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new(RateLimitConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_bucket_refill() {
        let mut bucket = TokenBucket::new(10.0, 1.0); // 1 token per second

        // Consume all tokens
        assert!(bucket.try_consume(10.0));
        assert!(!bucket.try_consume(1.0));

        // Wait for refill
        std::thread::sleep(Duration::from_millis(1100));
        assert!(bucket.try_consume(1.0));
    }

    #[tokio::test]
    async fn test_rate_limiter_basic() {
        let limiter = RateLimiter::new(RateLimitConfig {
            requests_per_minute: 60, // 1 per second
            tokens_per_minute: None,
        });

        // First acquire should succeed immediately
        let start = Instant::now();
        limiter.acquire(None).await;
        let elapsed = start.elapsed();
        assert!(elapsed < Duration::from_millis(100));

        // Rapid successive acquires should be rate limited
        limiter.acquire(None).await;
        // Should have waited approximately 1 second
    }

    #[tokio::test]
    async fn test_try_acquire() {
        let limiter = RateLimiter::new(RateLimitConfig {
            requests_per_minute: 1, // Very restrictive
            tokens_per_minute: None,
        });

        assert!(limiter.try_acquire(None).await);
        assert!(!limiter.try_acquire(None).await); // Should fail immediately
    }
}
