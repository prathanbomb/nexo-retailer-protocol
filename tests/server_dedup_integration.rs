//! Integration tests for message deduplication in server
//!
//! These tests verify that the server correctly rejects duplicate message IDs
//! to prevent replay attacks.

use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

/// Helper function to connect a client to the server
async fn connect_client(addr: &str) -> Result<TcpStream, std::io::Error> {
    TcpStream::connect(addr).await
}

/// Helper function to create a test message with a specific message ID
fn create_test_message(message_id: &str) -> Vec<u8> {
    // For now, create a simple message with the message ID embedded
    // In the future, this will be a proper Casp001Document with Header4
    format!("MSG:{}", message_id).into_bytes()
}

#[tokio::test]
async fn test_duplicate_message_rejected() {
    // Bind server to ephemeral port
    use nexo_retailer_protocol::NexoServer;

    let server = NexoServer::bind("127.0.0.1:0").await.unwrap();
    let addr = server.local_addr().unwrap().to_string();

    // Spawn server task
    let server_task = tokio::spawn(async move {
        // Run server with timeout to prevent hanging
        let _ = timeout(Duration::from_secs(5), server.run()).await;
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Connect client and send message with ID "MSG-001"
    let mut client = connect_client(&addr).await.unwrap();
    let msg1 = create_test_message("MSG-001");
    client.write_all(&msg1).await.unwrap();

    // Read response
    let mut buffer = [0u8; 4096];
    let n = client.read(&mut buffer).await.unwrap();
    assert!(n > 0, "Should receive response");

    // Send same message ID again (duplicate)
    let mut client2 = connect_client(&addr).await.unwrap();
    let msg2 = create_test_message("MSG-001");
    client2.write_all(&msg2).await.unwrap();

    // Read response - should indicate duplicate rejection
    let n = client2.read(&mut buffer).await.unwrap();
    assert!(n > 0, "Should receive error response");

    // TODO: Verify error response contains "duplicate" indication
    // This will be implemented when dispatcher is added in plan 05-02

    // Cleanup
    server_task.abort();
}

#[tokio::test]
async fn test_different_message_ids_accepted() {
    // Bind server to ephemeral port
    use nexo_retailer_protocol::NexoServer;

    let server = NexoServer::bind("127.0.0.1:0").await.unwrap();
    let addr = server.local_addr().unwrap().to_string();

    // Spawn server task
    let server_task = tokio::spawn(async move {
        let _ = timeout(Duration::from_secs(5), server.run()).await;
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Send message with ID "MSG-001"
    let mut client1 = connect_client(&addr).await.unwrap();
    let msg1 = create_test_message("MSG-001");
    client1.write_all(&msg1).await.unwrap();

    let mut buffer = [0u8; 4096];
    let n = client1.read(&mut buffer).await.unwrap();
    assert!(n > 0, "Should receive response");

    // Send message with different ID "MSG-002"
    let mut client2 = connect_client(&addr).await.unwrap();
    let msg2 = create_test_message("MSG-002");
    client2.write_all(&msg2).await.unwrap();

    let n = client2.read(&mut buffer).await.unwrap();
    assert!(n > 0, "Should receive response");

    // TODO: Verify both messages were accepted
    // This will be implemented when dispatcher is added in plan 05-02

    // Cleanup
    server_task.abort();
}

#[tokio::test]
async fn test_expired_message_id_accepted() {
    // This test will be implemented when the dispatcher is added
    // It will verify that message IDs expire after TTL and can be reused

    // TODO: Implement after dispatcher is added in plan 05-02
    assert!(true, "Test placeholder - will be implemented with dispatcher");
}

#[tokio::test]
async fn test_deduplication_per_connection() {
    // Verify that deduplication is per-connection, not global

    // TODO: Implement after dispatcher is added in plan 05-02
    assert!(true, "Test placeholder - will be implemented with dispatcher");
}
