use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use crate::config::ThrottleConfig;

#[derive(Debug, Clone)]
struct Throttle {
	requests_per_second: u32,
	requests_per_minute: u32,
	requests_per_hour: u32,
	requests_per_day: u32,
	last_reset_second: Instant,
	last_reset_minute: Instant,
	last_reset_hour: Instant,
	last_reset_day: Instant,
}

impl Default for Throttle {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            requests_per_second: 0,
            requests_per_minute: 0,
            requests_per_hour: 0,
            requests_per_day: 0,
            last_reset_second: now,
            last_reset_minute: now,
            last_reset_hour: now,
            last_reset_day: now,
        }
    }
}

impl Throttle {
	fn new() -> Self {
		let now = Instant::now();
		Throttle {
			requests_per_second: 0,
			requests_per_minute: 0,
			requests_per_hour: 0,
			requests_per_day: 0,
			last_reset_second: now,
			last_reset_minute: now,
			last_reset_hour: now,
			last_reset_day: now,
		}
	}

	fn reset_if_needed(&mut self) {
		let now = Instant::now();

		// Reset per-second counter
		if now.duration_since(self.last_reset_second) >= Duration::from_secs(1) {
			self.requests_per_second = 0;
			self.last_reset_second = now;
		}

		// Reset per-minute counter
		if now.duration_since(self.last_reset_minute) >= Duration::from_secs(60) {
			self.requests_per_minute = 0;
			self.last_reset_minute = now;
		}

		// Reset per-hour counter
		if now.duration_since(self.last_reset_hour) >= Duration::from_secs(3600) {
			self.requests_per_hour = 0;
			self.last_reset_hour = now;
		}

		// Reset per-day counter
		if now.duration_since(self.last_reset_day) >= Duration::from_secs(86400) {
			self.requests_per_day = 0;
			self.last_reset_day = now;
		}
	}

	fn increment_counters(&mut self) {
		self.requests_per_second += 1;
		self.requests_per_minute += 1;
		self.requests_per_hour += 1;
		self.requests_per_day += 1;
	}

	fn should_throttle(&self, config: &ThrottleConfig) -> Option<Duration> {
		let now = Instant::now();

		if let Some(max_per_second) = config.max_requests_per_second {
			if self.requests_per_second >= max_per_second {
				return Some(Duration::from_secs(1) - now.duration_since(self.last_reset_second));
			}
		}

		if let Some(max_per_minute) = config.max_requests_per_minute {
			if self.requests_per_minute >= max_per_minute {
				return Some(Duration::from_secs(60) - now.duration_since(self.last_reset_minute));
			}
		}

		if let Some(max_per_hour) = config.max_requests_per_hour {
			if self.requests_per_hour >= max_per_hour {
				return Some(Duration::from_secs(3600) - now.duration_since(self.last_reset_hour));
			}
		}

		if let Some(max_per_day) = config.max_requests_per_day {
			if self.requests_per_day >= max_per_day {
				return Some(Duration::from_secs(86400) - now.duration_since(self.last_reset_day));
			}
		}

		None
	}
}

#[derive(Debug, Default)]
pub struct ThrottleManager {
    inner: Arc<Mutex<Throttle>>,
    config: ThrottleConfig,
}

impl ThrottleManager {
    pub fn new(config: ThrottleConfig) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Throttle::new())),
            config,
        }
    }

    pub async fn check_throttle(&self) -> Option<Duration> {
        let mut throttle = self.inner.lock().await;
        throttle.reset_if_needed();
        throttle.should_throttle(&self.config)

    }

    pub async fn increment_counters(&self) {
            self.inner.lock().await.increment_counters();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_throttle_limits() {
        // Create config with low limits for testing
        let config = ThrottleConfig {
            max_requests_per_second: Some(2),
            max_requests_per_minute: Some(5), 
            max_requests_per_hour: Some(10),
            max_requests_per_day: Some(20),
        };

        let manager = ThrottleManager::new(config);

        // Should allow initial requests
        assert_eq!(manager.check_throttle().await, None);
        manager.increment_counters().await;
        assert_eq!(manager.check_throttle().await, None);
        manager.increment_counters().await;

        // Should throttle after hitting per-second limit
        assert!(manager.check_throttle().await.is_some());

        // Wait 1 second for per-second counter to reset
        sleep(Duration::from_secs(1)).await;
        
        // Should allow more requests
        assert_eq!(manager.check_throttle().await, None);
        
        // Add more requests to hit per-minute limit
        for _ in 0..5 {
            manager.increment_counters().await;
        }

        // Should throttle after hitting per-minute limit
        assert!(manager.check_throttle().await.is_some());
    }
}
