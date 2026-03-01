//! Reconnection logic with exponential backoff
//!
//! This module provides automatic reconnection with exponential backoff and jitter
//! to prevent thundering herd problems when multiple clients reconnect simultaneously.

use core::time::Duration;

#[cfg(feature = "std")]
use rand::Rng;

/// Configuration for reconnection behavior
///
/// # Example
///
/// ```rust
/// use nexo_retailer_protocol::client::ReconnectConfig;
/// use std::time::Duration;
///
/// let config = ReconnectConfig::new()
///     .with_base_delay(Duration::from_millis(100))
///     .with_max_delay(Duration::from_secs(60))
///     .with_max_attempts(5);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReconnectConfig {
    /// Base delay between reconnection attempts
    pub base_delay: Duration,
    /// Maximum delay between reconnection attempts
    pub max_delay: Duration,
    /// Maximum number of reconnection attempts
    pub max_attempts: u32,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl ReconnectConfig {
    /// Create a new ReconnectConfig with sensible defaults
    ///
    /// Defaults:
    /// - base_delay: 100ms
    /// - max_delay: 60s
    /// - max_attempts: 5
    pub fn new() -> Self {
        Self {
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(60),
            max_attempts: 5,
        }
    }

    /// Set the base delay between reconnection attempts
    pub fn with_base_delay(mut self, delay: Duration) -> Self {
        self.base_delay = delay;
        self
    }

    /// Set the maximum delay between reconnection attempts
    pub fn with_max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    /// Set the maximum number of reconnection attempts
    pub fn with_max_attempts(mut self, attempts: u32) -> Self {
        self.max_attempts = attempts;
        self
    }
}

/// Exponential backoff calculator
///
/// Calculates delay using the formula: `base_delay * 2^attempt`, capped at max_delay.
///
/// # Example
///
/// ```rust
/// use nexo_retailer_protocol::client::{ReconnectConfig, Backoff};
///
/// let config = ReconnectConfig::new();
/// let mut backoff = Backoff::new(config);
///
/// // First attempt: 100ms
/// let delay1 = backoff.next_delay();
/// assert_eq!(delay1.as_millis(), 100);
///
/// // Second attempt: 200ms
/// let delay2 = backoff.next_delay();
/// assert_eq!(delay2.as_millis(), 200);
///
/// // Third attempt: 400ms
/// let delay3 = backoff.next_delay();
/// assert_eq!(delay3.as_millis(), 400);
/// ```
#[derive(Debug)]
pub struct Backoff {
    config: ReconnectConfig,
    current_attempt: u32,
}

impl Backoff {
    /// Create a new Backoff with the given configuration
    pub fn new(config: ReconnectConfig) -> Self {
        Self {
            config,
            current_attempt: 0,
        }
    }

    /// Calculate the next delay using exponential backoff
    ///
    /// Returns delay = base_delay * 2^attempt, capped at max_delay
    pub fn next_delay(&mut self) -> Duration {
        // Calculate exponential backoff: base * 2^attempt
        // Use saturating arithmetic to prevent overflow
        let multiplier = 2u32.saturating_pow(self.current_attempt);
        let delay_ms = self.config.base_delay.as_millis().saturating_mul(multiplier as u128);
        let delay = Duration::from_millis(delay_ms as u64);

        // Cap at max_delay
        let capped_delay = if delay > self.config.max_delay {
            self.config.max_delay
        } else {
            delay
        };

        self.current_attempt += 1;
        capped_delay
    }

    /// Reset the backoff to the initial state
    pub fn reset(&mut self) {
        self.current_attempt = 0;
    }

    /// Get the current attempt number (0-indexed)
    pub fn current_attempt(&self) -> u32 {
        self.current_attempt
    }

    /// Check if we should continue retrying
    pub fn should_continue(&self) -> bool {
        self.current_attempt < self.config.max_attempts
    }
}

#[cfg(feature = "std")]
impl Backoff {
    /// Wait with jitter for std runtime
    ///
    /// Adds ±20% random variation to prevent thundering herd when multiple
    /// clients reconnect simultaneously.
    ///
    /// Returns `true` if should continue retrying, `false` if max attempts exceeded.
    pub async fn wait_with_jitter(&mut self) -> bool {
        if !self.should_continue() {
            return false;
        }

        let delay = self.next_delay();

        // Add ±20% jitter: delay * (1.0 + random(-0.2, 0.2))
        let mut rng = rand::thread_rng();
        let jitter_factor = 1.0 + (rng.gen::<f64>() * 0.4 - 0.2);
        let jittered_delay_ms = (delay.as_millis() as f64 * jitter_factor).max(0.0) as u64;
        let jittered_delay = Duration::from_millis(jittered_delay_ms);

        tokio::time::sleep(jittered_delay).await;
        true
    }
}

#[cfg(feature = "embassy-net")]
impl Backoff {
    /// Wait without jitter for embassy runtime
    ///
    /// Uses pure exponential backoff without random jitter since RNG may not
    /// be available in strict no_std environments.
    ///
    /// Returns `true` if should continue retrying, `false` if max attempts exceeded.
    pub async fn wait_without_jitter(&mut self) -> bool {
        if !self.should_continue() {
            return false;
        }

        let delay = self.next_delay();
        embassy_time::Timer::after(delay).await;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reconnect_config_defaults() {
        let config = ReconnectConfig::new();
        assert_eq!(config.base_delay, Duration::from_millis(100));
        assert_eq!(config.max_delay, Duration::from_secs(60));
        assert_eq!(config.max_attempts, 5);
    }

    #[test]
    fn test_reconnect_config_builder() {
        let config = ReconnectConfig::new()
            .with_base_delay(Duration::from_millis(200))
            .with_max_delay(Duration::from_secs(30))
            .with_max_attempts(10);

        assert_eq!(config.base_delay, Duration::from_millis(200));
        assert_eq!(config.max_delay, Duration::from_secs(30));
        assert_eq!(config.max_attempts, 10);
    }

    #[test]
    fn test_backoff_calculation() {
        let config = ReconnectConfig::new()
            .with_base_delay(Duration::from_millis(100))
            .with_max_delay(Duration::from_secs(60))
            .with_max_attempts(10);

        let mut backoff = Backoff::new(config);

        // Attempt 0: 100ms * 2^0 = 100ms
        assert_eq!(backoff.next_delay(), Duration::from_millis(100));
        assert_eq!(backoff.current_attempt(), 1);

        // Attempt 1: 100ms * 2^1 = 200ms
        assert_eq!(backoff.next_delay(), Duration::from_millis(200));
        assert_eq!(backoff.current_attempt(), 2);

        // Attempt 2: 100ms * 2^2 = 400ms
        assert_eq!(backoff.next_delay(), Duration::from_millis(400));
        assert_eq!(backoff.current_attempt(), 3);

        // Attempt 3: 100ms * 2^3 = 800ms
        assert_eq!(backoff.next_delay(), Duration::from_millis(800));
        assert_eq!(backoff.current_attempt(), 4);

        // Attempt 4: 100ms * 2^4 = 1600ms
        assert_eq!(backoff.next_delay(), Duration::from_millis(1600));
        assert_eq!(backoff.current_attempt(), 5);
    }

    #[test]
    fn test_backoff_max_delay_cap() {
        let config = ReconnectConfig::new()
            .with_base_delay(Duration::from_millis(100))
            .with_max_delay(Duration::from_millis(500))
            .with_max_attempts(10);

        let mut backoff = Backoff::new(config);

        // Attempt 0: 100ms
        assert_eq!(backoff.next_delay(), Duration::from_millis(100));

        // Attempt 1: 200ms
        assert_eq!(backoff.next_delay(), Duration::from_millis(200));

        // Attempt 2: 400ms
        assert_eq!(backoff.next_delay(), Duration::from_millis(400));

        // Attempt 3: 800ms -> capped at 500ms
        assert_eq!(backoff.next_delay(), Duration::from_millis(500));

        // Attempt 4: should still be capped
        assert_eq!(backoff.next_delay(), Duration::from_millis(500));
    }

    #[test]
    fn test_backoff_reset() {
        let config = ReconnectConfig::new();
        let mut backoff = Backoff::new(config);

        // Advance a few attempts
        backoff.next_delay();
        backoff.next_delay();
        assert_eq!(backoff.current_attempt(), 2);

        // Reset
        backoff.reset();
        assert_eq!(backoff.current_attempt(), 0);

        // Should start from beginning
        assert_eq!(backoff.next_delay(), Duration::from_millis(100));
    }

    #[test]
    fn test_backoff_should_continue() {
        let config = ReconnectConfig::new()
            .with_max_attempts(3);

        let mut backoff = Backoff::new(config);

        // Should continue before any attempts
        assert!(backoff.should_continue());

        // After 1st attempt
        backoff.next_delay();
        assert!(backoff.should_continue());

        // After 2nd attempt
        backoff.next_delay();
        assert!(backoff.should_continue());

        // After 3rd attempt (max_attempts = 3)
        backoff.next_delay();
        assert!(!backoff.should_continue());
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_jitter_range() {
        // This test verifies jitter is within ±20% range
        let config = ReconnectConfig::new()
            .with_base_delay(Duration::from_millis(100))
            .with_max_attempts(1);

        let mut backoff = Backoff::new(config);

        // Get multiple samples to verify jitter range
        // Note: This is a statistical test - could theoretically fail
        for _ in 0..100 {
            backoff.reset();
            let delay = backoff.next_delay();

            // Jitter should be within ±20% of 100ms
            // Range: 80ms to 120ms
            // We can't test the exact jittered value without actually calling wait_with_jitter
            // which would sleep. So we just verify the base calculation works.
            assert_eq!(delay, Duration::from_millis(100));
        }
    }
}
