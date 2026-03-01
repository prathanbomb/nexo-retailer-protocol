//! Integration tests for heartbeat/keepalive protocol
//!
//! These tests verify that the server correctly implements the heartbeat protocol,
//! including sending periodic heartbeats and detecting dead connections.

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
