use governor::{Quota, RateLimiter, Jitter};
use governor::clock::DefaultClock;
use governor::state::{InMemoryState, NotKeyed};
use std::sync::Arc;
use nonzero_ext::*;
use std::time::Duration;

#[derive(Clone)]
pub struct ApiRateLimiter {
    limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
    burst_limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
}

impl ApiRateLimiter {
    pub fn new() -> Self {
        let limiter = Arc::new(RateLimiter::direct(
            Quota::per_minute(nonzero!(50u32))
        ));

        let burst_limiter = Arc::new(RateLimiter::direct(
            Quota::per_second(nonzero!(5u32))
        ));

        Self {
            limiter,
            burst_limiter,
        }
    }

    pub async fn acquire(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.burst_limiter.until_ready().await;
        self.limiter.until_ready_with_jitter(Jitter::up_to(Duration::from_millis(100))).await;

        Ok(())
    }

    pub fn check_remaining(&self) -> u32 {
        match self.limiter.check() {
            Ok(_) => 1,
            Err(_) => 0,
        }
    }
}