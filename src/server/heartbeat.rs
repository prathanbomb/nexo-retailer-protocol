//! Heartbeat/keepalive protocol for dead connection detection
//!
//! This module provides a heartbeat protocol to detect dead connections faster
//! than TCP keepalive. The server sends periodic heartbeat messages and closes
//! connections that don't respond within a timeout. This uses `tokio::select!`
//! to concurrently handle incoming messages and heartbeat ticks.

use crate::error::NexoError;
use std::time::{Duration, Instant};

/// Default heartbeat interval (30 seconds)
const DEFAULT_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);

/// Default heartbeat timeout (90 seconds = 3x interval)
const DEFAULT_HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(90);

/// Heartbeat configuration
///
/// This struct defines the heartbeat parameters for a connection, including
/// the interval at which heartbeats are sent and the timeout after which a
/// connection is considered dead.
///
/// # Builder Pattern
///
/// Use `HeartbeatConfig::new()` to create a config with defaults, then use
/// builder methods to customize:
///
/// ```
/// use nexo_retailer_protocol::server::HeartbeatConfig;
/// use std::time::Duration;
///
/// let config = HeartbeatConfig::new()
///     .with_interval(Duration::from_secs(60))
///     .with_timeout(Duration::from_secs(180));
/// ```
///
/// # Validation
///
/// The config validates that timeout > interval on build. If this constraint
/// is violated, `build()` returns an error.
#[derive(Debug, Clone, PartialEq)]
pub struct HeartbeatConfig {
    /// Interval at which to send heartbeat messages
    interval: Duration,
    /// Timeout after which connection is considered dead
    timeout: Duration,
    /// Whether heartbeat monitoring is enabled
    enabled: bool,
}

impl HeartbeatConfig {
    /// Create a new heartbeat config with default values
    ///
    /// # Defaults
    ///
    /// - interval: 30 seconds
    /// - timeout: 90 seconds (3x interval)
    /// - enabled: true
    ///
    /// # Examples
    ///
    /// ```
    /// use nexo_retailer_protocol::server::HeartbeatConfig;
    ///
    /// let config = HeartbeatConfig::new();
    /// assert_eq!(config.interval(), std::time::Duration::from_secs(30));
    /// assert_eq!(config.timeout(), std::time::Duration::from_secs(90));
    /// assert!(config.is_enabled());
    /// ```
    pub fn new() -> Self {
        Self {
            interval: DEFAULT_HEARTBEAT_INTERVAL,
            timeout: DEFAULT_HEARTBEAT_TIMEOUT,
            enabled: true,
        }
    }

    /// Set the heartbeat interval
    ///
    /// # Arguments
    ///
    /// * `interval` - Time between heartbeat messages
    ///
    /// # Returns
    ///
    /// Self for builder pattern chaining
    ///
    /// # Examples
    ///
    /// ```
    /// use nexo_retailer_protocol::server::HeartbeatConfig;
    /// use std::time::Duration;
    ///
    /// let config = HeartbeatConfig::new()
    ///     .with_interval(Duration::from_secs(60));
    /// assert_eq!(config.interval(), Duration::from_secs(60));
    /// ```
    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }

    /// Set the heartbeat timeout
    ///
    /// # Arguments
    ///
    /// * `timeout` - Time after which connection is considered dead
    ///
    /// # Returns
    ///
    /// Self for builder pattern chaining
    ///
    /// # Examples
    ///
    /// ```
    /// use nexo_retailer_protocol::server::HeartbeatConfig;
    /// use std::time::Duration;
    ///
    /// let config = HeartbeatConfig::new()
    ///     .with_timeout(Duration::from_secs(180));
    /// assert_eq!(config.timeout(), Duration::from_secs(180));
    /// ```
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Enable or disable heartbeat monitoring
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether to enable heartbeat monitoring
    ///
    /// # Returns
    ///
    /// Self for builder pattern chaining
    ///
    /// # Examples
    ///
    /// ```
    /// use nexo_retailer_protocol::server::HeartbeatConfig;
    ///
    /// let config = HeartbeatConfig::new()
    ///     .with_enabled(false);
    /// assert!(!config.is_enabled());
    /// ```
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Build and validate the heartbeat config
    ///
    /// This method validates that timeout > interval. If this constraint
    /// is violated, it returns an error.
    ///
    /// # Returns
    ///
    /// * `Ok(Self)` - If the config is valid
    /// * `Err(NexoError::Validation)` - If timeout <= interval
    ///
    /// # Examples
    ///
    /// ```
    /// use nexo_retailer_protocol::server::HeartbeatConfig;
    /// use std::time::Duration;
    ///
    /// // Valid config
    /// let config = HeartbeatConfig::new()
    ///     .with_interval(Duration::from_secs(30))
    ///     .with_timeout(Duration::from_secs(90));
    /// assert!(config.build().is_ok());
    ///
    /// // Invalid config (timeout < interval)
    /// let config = HeartbeatConfig::new()
    ///     .with_interval(Duration::from_secs(90))
    ///     .with_timeout(Duration::from_secs(30));
    /// assert!(config.build().is_err());
    /// ```
    pub fn build(self) -> Result<Self, NexoError> {
        // Validate that timeout > interval
        if self.timeout <= self.interval {
            return Err(NexoError::Validation {
                field: "heartbeat_timeout",
                reason: "timeout must be greater than interval",
            });
        }

        Ok(self)
    }

    /// Get the heartbeat interval
    ///
    /// # Returns
    ///
    /// The time between heartbeat messages
    pub fn interval(&self) -> Duration {
        self.interval
    }

    /// Get the heartbeat timeout
    ///
    /// # Returns
    ///
    /// The time after which connection is considered dead
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Check if heartbeat monitoring is enabled
    ///
    /// # Returns
    ///
    /// `true` if enabled, `false` otherwise
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Default for HeartbeatConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Heartbeat monitor for tracking connection activity
///
/// This monitor tracks the last activity timestamp and checks for timeout
/// conditions. It is used within the connection loop to detect dead connections.
///
/// # Thread Safety
///
/// This type is NOT thread-safe by itself. It should be protected by a Mutex
/// when used in concurrent contexts (e.g., within `ConnectionState` which is
/// already wrapped in `Arc<Mutex<>>` in the server).
///
/// # Examples
///
/// ```
/// use nexo_retailer_protocol::server::heartbeat::{HeartbeatConfig, HeartbeatMonitor};
/// use std::time::Duration;
///
/// let config = HeartbeatConfig::new();
/// let mut monitor = HeartbeatMonitor::new(config);
///
/// // Update activity on message received
/// monitor.update_activity();
///
/// // Check if connection is dead
/// assert!(!monitor.check_timeout());
///
/// // Check if heartbeat should be sent
/// // (This would be checked periodically in the connection loop)
/// ```
#[derive(Debug, Clone)]
pub struct HeartbeatMonitor {
    /// Heartbeat configuration
    config: HeartbeatConfig,
    /// Last activity timestamp
    last_activity: Instant,
    /// Last heartbeat timestamp
    last_heartbeat: Instant,
}

impl HeartbeatMonitor {
    /// Create a new heartbeat monitor
    ///
    /// # Arguments
    ///
    /// * `config` - Heartbeat configuration
    ///
    /// # Examples
    ///
    /// ```
    /// use nexo_retailer_protocol::server::heartbeat::{HeartbeatConfig, HeartbeatMonitor};
    ///
    /// let config = HeartbeatConfig::new();
    /// let monitor = HeartbeatMonitor::new(config);
    /// ```
    pub fn new(config: HeartbeatConfig) -> Self {
        let now = Instant::now();
        Self {
            config,
            last_activity: now,
            last_heartbeat: now,
        }
    }

    /// Update the activity timestamp (called when message received)
    ///
    /// This should be called whenever a message is received from the client,
    /// indicating that the connection is still alive.
    ///
    /// # Examples
    ///
    /// ```
    /// use nexo_retailer_protocol::server::heartbeat::{HeartbeatConfig, HeartbeatMonitor};
    ///
    /// let config = HeartbeatConfig::new();
    /// let mut monitor = HeartbeatMonitor::new(config);
    ///
    /// // Simulate receiving a message
    /// monitor.update_activity();
    /// ```
    pub fn update_activity(&mut self) {
        self.last_activity = Instant::now();
    }

    /// Check if the connection has timed out
    ///
    /// Returns `true` if the time since last activity exceeds the configured
    /// timeout, indicating that the connection is dead.
    ///
    /// # Returns
    ///
    /// `true` if connection is dead, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use nexo_retailer_protocol::server::heartbeat::{HeartbeatConfig, HeartbeatMonitor};
    /// use std::time::Duration;
    ///
    /// let config = HeartbeatConfig::new()
    ///     .with_timeout(Duration::from_millis(100));
    /// let mut monitor = HeartbeatMonitor::new(config);
    ///
    /// // Initially not timed out
    /// assert!(!monitor.check_timeout());
    ///
    /// // Wait for timeout
    /// std::thread::sleep(Duration::from_millis(150));
    /// assert!(monitor.check_timeout());
    /// ```
    pub fn check_timeout(&self) -> bool {
        if !self.config.is_enabled() {
            return false;
        }

        let elapsed = self.last_activity.elapsed();
        elapsed > self.config.timeout()
    }

    /// Check if a heartbeat should be sent
    ///
    /// Returns `true` if the time since last heartbeat exceeds the configured
    /// interval, indicating that a heartbeat message should be sent.
    ///
    /// # Returns
    ///
    /// `true` if heartbeat should be sent, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use nexo_retailer_protocol::server::heartbeat::{HeartbeatConfig, HeartbeatMonitor};
    /// use std::time::Duration;
    ///
    /// let config = HeartbeatConfig::new()
    ///     .with_interval(Duration::from_millis(100));
    /// let mut monitor = HeartbeatMonitor::new(config);
    ///
    /// // Initially no heartbeat needed
    /// assert!(!monitor.should_send_heartbeat());
    ///
    /// // Wait for interval
    /// std::thread::sleep(Duration::from_millis(150));
    /// assert!(monitor.should_send_heartbeat());
    /// ```
    pub fn should_send_heartbeat(&self) -> bool {
        if !self.config.is_enabled() {
            return false;
        }

        let elapsed = self.last_heartbeat.elapsed();
        elapsed >= self.config.interval()
    }

    /// Mark that a heartbeat was sent
    ///
    /// This should be called after sending a heartbeat message to update
    /// the last heartbeat timestamp.
    ///
    /// # Examples
    ///
    /// ```
    /// use nexo_retailer_protocol::server::heartbeat::{HeartbeatConfig, HeartbeatMonitor};
    /// use std::time::Duration;
    ///
    /// let config = HeartbeatConfig::new()
    ///     .with_interval(Duration::from_millis(100));
    /// let mut monitor = HeartbeatMonitor::new(config);
    ///
    /// // Wait for interval
    /// std::thread::sleep(Duration::from_millis(150));
    /// assert!(monitor.should_send_heartbeat());
    ///
    /// // Mark heartbeat as sent
    /// monitor.mark_heartbeat_sent();
    /// assert!(!monitor.should_send_heartbeat());
    /// ```
    pub fn mark_heartbeat_sent(&mut self) {
        self.last_heartbeat = Instant::now();
    }

    /// Get the time since last activity
    ///
    /// # Returns
    ///
    /// The duration since the last activity
    pub fn time_since_activity(&self) -> Duration {
        self.last_activity.elapsed()
    }

    /// Get the heartbeat configuration
    ///
    /// # Returns
    ///
    /// The heartbeat configuration
    pub fn config(&self) -> &HeartbeatConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heartbeat_config_new_has_defaults() {
        let config = HeartbeatConfig::new();
        assert_eq!(config.interval(), DEFAULT_HEARTBEAT_INTERVAL);
        assert_eq!(config.timeout(), DEFAULT_HEARTBEAT_TIMEOUT);
        assert!(config.is_enabled());
    }

    #[test]
    fn test_heartbeat_config_default() {
        let config = HeartbeatConfig::default();
        assert_eq!(config.interval(), DEFAULT_HEARTBEAT_INTERVAL);
        assert_eq!(config.timeout(), DEFAULT_HEARTBEAT_TIMEOUT);
        assert!(config.is_enabled());
    }

    #[test]
    fn test_heartbeat_config_with_interval() {
        let config = HeartbeatConfig::new()
            .with_interval(Duration::from_secs(60));
        assert_eq!(config.interval(), Duration::from_secs(60));
    }

    #[test]
    fn test_heartbeat_config_with_timeout() {
        let config = HeartbeatConfig::new()
            .with_timeout(Duration::from_secs(180));
        assert_eq!(config.timeout(), Duration::from_secs(180));
    }

    #[test]
    fn test_heartbeat_config_with_enabled() {
        let config = HeartbeatConfig::new()
            .with_enabled(false);
        assert!(!config.is_enabled());
    }

    #[test]
    fn test_heartbeat_config_builder_chain() {
        let config = HeartbeatConfig::new()
            .with_interval(Duration::from_secs(45))
            .with_timeout(Duration::from_secs(135))
            .with_enabled(true);

        assert_eq!(config.interval(), Duration::from_secs(45));
        assert_eq!(config.timeout(), Duration::from_secs(135));
        assert!(config.is_enabled());
    }

    #[test]
    fn test_heartbeat_config_build_valid() {
        let config = HeartbeatConfig::new()
            .with_interval(Duration::from_secs(30))
            .with_timeout(Duration::from_secs(90));

        let result = config.build();
        assert!(result.is_ok());

        let built = result.unwrap();
        assert_eq!(built.interval(), Duration::from_secs(30));
        assert_eq!(built.timeout(), Duration::from_secs(90));
    }

    #[test]
    fn test_heartbeat_config_build_timeout_equals_interval() {
        let config = HeartbeatConfig::new()
            .with_interval(Duration::from_secs(30))
            .with_timeout(Duration::from_secs(30));

        let result = config.build();
        assert!(result.is_err());

        match result {
            Err(NexoError::Validation { field, reason }) => {
                assert_eq!(field, "heartbeat_timeout");
                assert!(reason.contains("timeout must be greater than interval"));
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_heartbeat_config_build_timeout_less_than_interval() {
        let config = HeartbeatConfig::new()
            .with_interval(Duration::from_secs(90))
            .with_timeout(Duration::from_secs(30));

        let result = config.build();
        assert!(result.is_err());

        match result {
            Err(NexoError::Validation { field, .. }) => {
                assert_eq!(field, "heartbeat_timeout");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_heartbeat_monitor_new() {
        let config = HeartbeatConfig::new();
        let monitor = HeartbeatMonitor::new(config);

        assert!(!monitor.check_timeout());
        assert!(!monitor.should_send_heartbeat());
        assert_eq!(monitor.time_since_activity().as_secs(), 0);
    }

    #[test]
    fn test_heartbeat_monitor_update_activity() {
        let config = HeartbeatConfig::new();
        let mut monitor = HeartbeatMonitor::new(config);

        // Wait a bit
        std::thread::sleep(Duration::from_millis(10));

        // Update activity
        monitor.update_activity();

        // Time since activity should be small
        assert!(monitor.time_since_activity() < Duration::from_millis(10));
    }

    #[test]
    fn test_heartbeat_monitor_check_timeout_not_expired() {
        let config = HeartbeatConfig::new()
            .with_timeout(Duration::from_secs(1));
        let mut monitor = HeartbeatMonitor::new(config);

        // Update activity
        monitor.update_activity();

        // Should not timeout immediately
        assert!(!monitor.check_timeout());

        // Still not timed out after 500ms
        std::thread::sleep(Duration::from_millis(500));
        assert!(!monitor.check_timeout());
    }

    #[test]
    fn test_heartbeat_monitor_check_timeout_expired() {
        let config = HeartbeatConfig::new()
            .with_timeout(Duration::from_millis(100));
        let mut monitor = HeartbeatMonitor::new(config);

        // Initially not timed out
        assert!(!monitor.check_timeout());

        // Wait for timeout
        std::thread::sleep(Duration::from_millis(150));

        // Should be timed out
        assert!(monitor.check_timeout());
    }

    #[test]
    fn test_heartbeat_monitor_check_timeout_reset_on_activity() {
        let config = HeartbeatConfig::new()
            .with_timeout(Duration::from_millis(100));
        let mut monitor = HeartbeatMonitor::new(config);

        // Wait 50ms
        std::thread::sleep(Duration::from_millis(50));
        assert!(!monitor.check_timeout());

        // Update activity (resets timeout)
        monitor.update_activity();

        // Wait another 50ms
        std::thread::sleep(Duration::from_millis(50));
        assert!(!monitor.check_timeout());

        // Wait another 50ms (total 100ms since last activity)
        std::thread::sleep(Duration::from_millis(50));
        assert!(monitor.check_timeout());
    }

    #[test]
    fn test_heartbeat_monitor_should_send_heartbeat_initially_false() {
        let config = HeartbeatConfig::new()
            .with_interval(Duration::from_millis(100));
        let monitor = HeartbeatMonitor::new(config);

        // Initially no heartbeat needed
        assert!(!monitor.should_send_heartbeat());
    }

    #[test]
    fn test_heartbeat_monitor_should_send_heartbeat_after_interval() {
        let config = HeartbeatConfig::new()
            .with_interval(Duration::from_millis(100));
        let monitor = HeartbeatMonitor::new(config);

        // Wait for interval
        std::thread::sleep(Duration::from_millis(150));

        // Should send heartbeat
        assert!(monitor.should_send_heartbeat());
    }

    #[test]
    fn test_heartbeat_monitor_mark_heartbeat_sent() {
        let config = HeartbeatConfig::new()
            .with_interval(Duration::from_millis(100));
        let mut monitor = HeartbeatMonitor::new(config);

        // Wait for interval
        std::thread::sleep(Duration::from_millis(150));
        assert!(monitor.should_send_heartbeat());

        // Mark as sent
        monitor.mark_heartbeat_sent();

        // Should not need another heartbeat immediately
        assert!(!monitor.should_send_heartbeat());
    }

    #[test]
    fn test_heartbeat_monitor_disabled_never_times_out() {
        let config = HeartbeatConfig::new()
            .with_timeout(Duration::from_millis(10))
            .with_enabled(false);
        let mut monitor = HeartbeatMonitor::new(config);

        // Wait past timeout
        std::thread::sleep(Duration::from_millis(50));

        // Should not timeout when disabled
        assert!(!monitor.check_timeout());
    }

    #[test]
    fn test_heartbeat_monitor_disabled_never_sends_heartbeat() {
        let config = HeartbeatConfig::new()
            .with_interval(Duration::from_millis(10))
            .with_enabled(false);
        let monitor = HeartbeatMonitor::new(config);

        // Wait past interval
        std::thread::sleep(Duration::from_millis(50));

        // Should not send heartbeat when disabled
        assert!(!monitor.should_send_heartbeat());
    }

    #[test]
    fn test_heartbeat_monitor_config_accessor() {
        let config = HeartbeatConfig::new()
            .with_interval(Duration::from_secs(60))
            .with_timeout(Duration::from_secs(180));
        let monitor = HeartbeatMonitor::new(config);

        let retrieved = monitor.config();
        assert_eq!(retrieved.interval(), Duration::from_secs(60));
        assert_eq!(retrieved.timeout(), Duration::from_secs(180));
    }

    #[test]
    fn test_heartbeat_monitor_time_since_activity_increases() {
        let config = HeartbeatConfig::new();
        let monitor = HeartbeatMonitor::new(config);

        let time1 = monitor.time_since_activity();
        std::thread::sleep(Duration::from_millis(10));
        let time2 = monitor.time_since_activity();

        assert!(time2 > time1);
    }
}
