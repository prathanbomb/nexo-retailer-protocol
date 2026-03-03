//! Integration tests for heartbeat/keepalive protocol
//!
//! These tests verify that the server correctly implements the heartbeat protocol,
//! including sending periodic heartbeats and detecting dead connections.

#![cfg(feature = "std")]

use std::sync::Arc;
use std::time::Duration;

use nexo_retailer_protocol::NexoServer;
use nexo_retailer_protocol::server::{RequestHandler, HeartbeatConfig};

/// Mock handler for testing
struct MockHandler;

#[async_trait::async_trait]
impl RequestHandler for MockHandler {
    async fn handle_payment_request(
        &self,
        _request: nexo_retailer_protocol::Casp001Document,
    ) -> Result<nexo_retailer_protocol::Casp002Document, nexo_retailer_protocol::NexoError> {
        // Return default response
        Ok(nexo_retailer_protocol::Casp002Document::default())
    }

    async fn handle_admin_request(
        &self,
        _request: nexo_retailer_protocol::Casp003Document,
    ) -> Result<nexo_retailer_protocol::Casp004Document, nexo_retailer_protocol::NexoError> {
        // Return default response
        Ok(nexo_retailer_protocol::Casp004Document::default())
    }
}

#[tokio::test]
async fn test_heartbeat_config_default_values() {
    let config = HeartbeatConfig::new();
    assert_eq!(config.interval(), Duration::from_secs(30));
    assert_eq!(config.timeout(), Duration::from_secs(90));
    assert!(config.is_enabled());
}

#[tokio::test]
async fn test_heartbeat_config_custom_values() {
    let config = HeartbeatConfig::new()
        .with_interval(Duration::from_secs(60))
        .with_timeout(Duration::from_secs(180));

    assert_eq!(config.interval(), Duration::from_secs(60));
    assert_eq!(config.timeout(), Duration::from_secs(180));
}

#[tokio::test]
async fn test_heartbeat_config_build_valid() {
    let config = HeartbeatConfig::new()
        .with_interval(Duration::from_secs(30))
        .with_timeout(Duration::from_secs(90));

    let result = config.build();
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_heartbeat_config_build_invalid_timeout_equals_interval() {
    let config = HeartbeatConfig::new()
        .with_interval(Duration::from_secs(30))
        .with_timeout(Duration::from_secs(30));

    let result = config.build();
    assert!(result.is_err());
}

#[tokio::test]
async fn test_heartbeat_config_build_invalid_timeout_less_than_interval() {
    let config = HeartbeatConfig::new()
        .with_interval(Duration::from_secs(90))
        .with_timeout(Duration::from_secs(30));

    let result = config.build();
    assert!(result.is_err());
}

#[tokio::test]
async fn test_server_with_heartbeat_config() {
    // Bind server to ephemeral port
    let server = NexoServer::bind("127.0.0.1:0").await.unwrap();
    let addr = server.local_addr().unwrap();

    // Verify server is bound
    assert_eq!(addr.ip(), std::net::Ipv4Addr::new(127, 0, 0, 1));
    assert!(addr.port() > 0);
}

#[tokio::test]
async fn test_server_connection_with_heartbeat_handler() {
    // Bind server to ephemeral port
    let server = NexoServer::bind("127.0.0.1:0").await.unwrap();
    let addr = server.local_addr().unwrap().to_string();

    // Set up handler
    let handler = Arc::new(MockHandler);
    let server = server.with_handler(handler);

    // Spawn server task
    let server_task = tokio::spawn(async move {
        // Accept one connection (will timeout after 1 second)
        tokio::time::timeout(Duration::from_secs(1), server.run()).await
    });

    // Connect a client
    let client = tokio::net::TcpStream::connect(&addr).await;
    assert!(client.is_ok(), "Client should connect successfully");

    // Wait a bit to ensure connection is established
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Abort server task
    server_task.abort();

    // Verify connection count
    // Note: We can't easily check this without more complex coordination
}

#[tokio::test]
async fn test_heartbeat_connection_state_tracking() {
    use nexo_retailer_protocol::server::ConnectionState;

    let addr = "127.0.0.1:8080".parse().unwrap();
    let mut state = ConnectionState::new(addr);

    // Initially, last_activity should be very recent
    let elapsed = state.last_activity().elapsed();
    assert!(elapsed < Duration::from_secs(1));

    // Update activity
    state.update_activity();

    // After update, elapsed should be very small
    let elapsed = state.last_activity().elapsed();
    assert!(elapsed < Duration::from_millis(10));
}

#[tokio::test]
async fn test_heartbeat_connection_state_with_custom_config() {
    use nexo_retailer_protocol::server::ConnectionState;

    let addr = "127.0.0.1:8080".parse().unwrap();
    let mut state = ConnectionState::new(addr);

    // Set custom heartbeat config
    let config = HeartbeatConfig::new()
        .with_interval(Duration::from_secs(60))
        .with_timeout(Duration::from_secs(180));

    state.set_heartbeat_config(Some(config));

    // Verify config is set
    assert!(state.heartbeat_config().is_some());
    let retrieved = state.heartbeat_config().unwrap();
    assert_eq!(retrieved.interval(), Duration::from_secs(60));
    assert_eq!(retrieved.timeout(), Duration::from_secs(180));
}

#[tokio::test]
async fn test_heartbeat_monitor_timeout_detection() {
    use nexo_retailer_protocol::server::heartbeat::{HeartbeatConfig, HeartbeatMonitor};

    let config = HeartbeatConfig::new()
        .with_timeout(Duration::from_millis(100));
    let mut monitor = HeartbeatMonitor::new(config);

    // Initially not timed out
    assert!(!monitor.check_timeout());

    // Wait for timeout
    tokio::time::sleep(Duration::from_millis(150)).await;

    // Should be timed out
    assert!(monitor.check_timeout());
}

#[tokio::test]
async fn test_heartbeat_monitor_activity_reset() {
    use nexo_retailer_protocol::server::heartbeat::{HeartbeatConfig, HeartbeatMonitor};

    let config = HeartbeatConfig::new()
        .with_timeout(Duration::from_millis(100));
    let mut monitor = HeartbeatMonitor::new(config);

    // Wait 50ms
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert!(!monitor.check_timeout());

    // Update activity (resets timeout)
    monitor.update_activity();

    // Wait another 50ms (total 100ms from start, but only 50ms from last activity)
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert!(!monitor.check_timeout());

    // Wait another 60ms (total 110ms from last activity)
    tokio::time::sleep(Duration::from_millis(60)).await;
    assert!(monitor.check_timeout());
}

#[tokio::test]
async fn test_heartbeat_monitor_interval_triggering() {
    use nexo_retailer_protocol::server::heartbeat::{HeartbeatConfig, HeartbeatMonitor};

    let config = HeartbeatConfig::new()
        .with_interval(Duration::from_millis(100));
    let monitor = HeartbeatMonitor::new(config);

    // Initially no heartbeat needed
    assert!(!monitor.should_send_heartbeat());

    // Wait for interval
    tokio::time::sleep(Duration::from_millis(150)).await;

    // Should send heartbeat
    assert!(monitor.should_send_heartbeat());
}

#[tokio::test]
async fn test_heartbeat_monitor_mark_sent() {
    use nexo_retailer_protocol::server::heartbeat::{HeartbeatConfig, HeartbeatMonitor};

    let config = HeartbeatConfig::new()
        .with_interval(Duration::from_millis(100));
    let monitor = HeartbeatMonitor::new(config);

    // Wait for interval
    tokio::time::sleep(Duration::from_millis(150)).await;
    assert!(monitor.should_send_heartbeat());

    // Mark as sent
    let mut monitor = monitor;
    monitor.mark_heartbeat_sent();

    // Should not need another heartbeat immediately
    assert!(!monitor.should_send_heartbeat());
}

#[tokio::test]
async fn test_heartbeat_monitor_disabled() {
    use nexo_retailer_protocol::server::heartbeat::{HeartbeatConfig, HeartbeatMonitor};

    let config = HeartbeatConfig::new()
        .with_interval(Duration::from_millis(10))
        .with_timeout(Duration::from_millis(10))
        .with_enabled(false);
    let monitor = HeartbeatMonitor::new(config);

    // Wait past both interval and timeout
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Should not timeout when disabled
    assert!(!monitor.check_timeout());

    // Should not send heartbeat when disabled
    assert!(!monitor.should_send_heartbeat());
}

// ============================================================================
// Task 4: Extended Heartbeat Protocol Tests
// ============================================================================

#[tokio::test]
async fn test_heartbeat_interval_timing() {
    // Test that heartbeat interval timing is correct
    use nexo_retailer_protocol::server::heartbeat::{HeartbeatConfig, HeartbeatMonitor};

    // Test with 50ms interval
    let config = HeartbeatConfig::new()
        .with_interval(Duration::from_millis(50))
        .with_timeout(Duration::from_millis(200));
    let monitor = HeartbeatMonitor::new(config);

    // Initially no heartbeat needed
    assert!(!monitor.should_send_heartbeat(), "No heartbeat should be needed initially");

    // Wait 40ms (less than interval)
    tokio::time::sleep(Duration::from_millis(40)).await;
    assert!(!monitor.should_send_heartbeat(), "No heartbeat before interval");

    // Wait another 20ms (total 60ms, past interval)
    tokio::time::sleep(Duration::from_millis(20)).await;
    assert!(monitor.should_send_heartbeat(), "Heartbeat should be needed after interval");
}

#[tokio::test]
async fn test_heartbeat_timeout_detection_timing() {
    // Test that timeout detection works with precise timing
    use nexo_retailer_protocol::server::heartbeat::{HeartbeatConfig, HeartbeatMonitor};

    // Test with 100ms timeout
    let config = HeartbeatConfig::new()
        .with_interval(Duration::from_millis(30))
        .with_timeout(Duration::from_millis(100));
    let monitor = HeartbeatMonitor::new(config);

    // Not timed out initially
    assert!(!monitor.check_timeout(), "Should not be timed out initially");

    // Wait 50ms (less than timeout)
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert!(!monitor.check_timeout(), "Should not be timed out at 50ms");

    // Wait another 60ms (total 110ms, past timeout)
    tokio::time::sleep(Duration::from_millis(60)).await;
    assert!(monitor.check_timeout(), "Should be timed out after 110ms");
}

#[tokio::test]
async fn test_heartbeat_missing_response_handling() {
    // Test that heartbeat monitor handles missing responses correctly
    use nexo_retailer_protocol::server::heartbeat::{HeartbeatConfig, HeartbeatMonitor};

    let config = HeartbeatConfig::new()
        .with_interval(Duration::from_millis(50))
        .with_timeout(Duration::from_millis(150));
    let mut monitor = HeartbeatMonitor::new(config);

    // Simulate heartbeat sent but no response
    tokio::time::sleep(Duration::from_millis(60)).await;
    assert!(monitor.should_send_heartbeat(), "Should need heartbeat after interval");
    monitor.mark_heartbeat_sent();

    // Wait for timeout (simulating no response)
    tokio::time::sleep(Duration::from_millis(160)).await;

    // Should timeout due to missing response
    assert!(monitor.check_timeout(), "Should timeout when no heartbeat response");
}

#[tokio::test]
async fn test_heartbeat_disabled_connection_works() {
    // Test that connections work without heartbeat when disabled
    use nexo_retailer_protocol::server::heartbeat::{HeartbeatConfig, HeartbeatMonitor};

    // Create disabled config
    let config = HeartbeatConfig::new()
        .with_interval(Duration::from_millis(10))
        .with_timeout(Duration::from_millis(30))
        .with_enabled(false);
    let mut monitor = HeartbeatMonitor::new(config);

    // Wait longer than both interval and timeout
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Should not require heartbeat
    assert!(!monitor.should_send_heartbeat(), "Disabled heartbeat should not require sending");

    // Should not detect timeout
    assert!(!monitor.check_timeout(), "Disabled heartbeat should not detect timeout");

    // Activity update should still work
    monitor.update_activity();
    assert!(!monitor.check_timeout(), "Still no timeout after activity update when disabled");
}

#[tokio::test]
async fn test_heartbeat_configurable_intervals() {
    // Test that custom intervals work correctly
    use nexo_retailer_protocol::server::heartbeat::{HeartbeatConfig, HeartbeatMonitor};

    // Test with custom short interval
    let short_config = HeartbeatConfig::new()
        .with_interval(Duration::from_millis(25))
        .with_timeout(Duration::from_millis(100));
    let short_monitor = HeartbeatMonitor::new(short_config);

    tokio::time::sleep(Duration::from_millis(30)).await;
    assert!(short_monitor.should_send_heartbeat(), "Short interval should trigger quickly");

    // Test with custom long interval
    let long_config = HeartbeatConfig::new()
        .with_interval(Duration::from_millis(200))
        .with_timeout(Duration::from_millis(600));
    let long_monitor = HeartbeatMonitor::new(long_config);

    tokio::time::sleep(Duration::from_millis(50)).await;
    assert!(!long_monitor.should_send_heartbeat(), "Long interval should not trigger yet");
}

#[tokio::test]
async fn test_heartbeat_3_to_1_ratio() {
    // Test that the 3:1 timeout-to-interval ratio is maintained
    // Default: 90s timeout, 30s interval

    let config = HeartbeatConfig::new();
    assert_eq!(config.timeout(), config.interval() * 3, "Timeout should be 3x interval");

    // Test with custom values maintaining ratio
    let custom_config = HeartbeatConfig::new()
        .with_interval(Duration::from_secs(60))
        .with_timeout(Duration::from_secs(180));
    assert_eq!(custom_config.timeout(), custom_config.interval() * 3, "Custom config should maintain 3:1 ratio");
}

#[tokio::test]
async fn test_heartbeat_multiple_cycles() {
    // Test heartbeat behavior over multiple cycles
    use nexo_retailer_protocol::server::heartbeat::{HeartbeatConfig, HeartbeatMonitor};

    let config = HeartbeatConfig::new()
        .with_interval(Duration::from_millis(30))
        .with_timeout(Duration::from_millis(100));
    let mut monitor = HeartbeatMonitor::new(config);

    // First cycle
    tokio::time::sleep(Duration::from_millis(35)).await;
    assert!(monitor.should_send_heartbeat(), "First cycle: should need heartbeat");
    monitor.mark_heartbeat_sent();
    monitor.update_activity();

    // Second cycle
    tokio::time::sleep(Duration::from_millis(35)).await;
    assert!(monitor.should_send_heartbeat(), "Second cycle: should need heartbeat");
    monitor.mark_heartbeat_sent();
    monitor.update_activity();

    // Third cycle
    tokio::time::sleep(Duration::from_millis(35)).await;
    assert!(monitor.should_send_heartbeat(), "Third cycle: should need heartbeat");

    // Should not be timed out because we kept updating activity
    assert!(!monitor.check_timeout(), "Should not timeout with regular activity");
}

#[tokio::test]
async fn test_heartbeat_server_connection_with_custom_config() {
    // Test that server connections can use custom heartbeat config
    use nexo_retailer_protocol::server::ConnectionState;

    let addr = "127.0.0.1:8080".parse().unwrap();
    let mut state = ConnectionState::new(addr);

    // Set a custom config
    let custom_config = HeartbeatConfig::new()
        .with_interval(Duration::from_secs(15))
        .with_timeout(Duration::from_secs(45));

    state.set_heartbeat_config(Some(custom_config));

    // Verify the config is retrievable and correct
    let retrieved = state.heartbeat_config();
    assert!(retrieved.is_some(), "Config should be set");
    let config = retrieved.unwrap();
    assert_eq!(config.interval(), Duration::from_secs(15));
    assert_eq!(config.timeout(), Duration::from_secs(45));
}

#[tokio::test]
async fn test_heartbeat_activity_update_prevents_timeout() {
    // Test that activity updates prevent timeout
    use nexo_retailer_protocol::server::heartbeat::{HeartbeatConfig, HeartbeatMonitor};

    let config = HeartbeatConfig::new()
        .with_interval(Duration::from_millis(30))
        .with_timeout(Duration::from_millis(90));
    let mut monitor = HeartbeatMonitor::new(config);

    // Repeatedly update activity to prevent timeout
    for _ in 0..5 {
        tokio::time::sleep(Duration::from_millis(40)).await;
        monitor.update_activity();
        assert!(!monitor.check_timeout(), "Should not timeout with activity updates");
    }

    // Stop updating and let it timeout
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(monitor.check_timeout(), "Should timeout without activity updates");
}
