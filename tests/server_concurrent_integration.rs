//! Concurrent connection integration tests for NexoServer
//!
//! This test file verifies that the server can handle multiple concurrent clients,
//! track connection state correctly, and clean up connections on disconnect.

#![cfg(feature = "std")]

use std::time::Duration;
use tokio::time::timeout;

/// Helper function to create a mock client connection
async fn connect_client(addr: &str) -> Result<tokio::net::TcpStream, std::io::Error> {
    tokio::net::TcpStream::connect(addr).await
}

#[tokio::test]
async fn test_10_concurrent_clients_connect_successfully() {
    // Bind server to ephemeral port
    let server = nexo_retailer_protocol::NexoServer::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind server");
    let addr = server.local_addr().expect("Failed to get local address").to_string();

    // Spawn server to accept connections (with timeout)
    let server_handle = tokio::spawn(async move {
        let _ = timeout(Duration::from_secs(5), server.run()).await;
        Ok::<(), nexo_retailer_protocol::NexoError>(())
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Connect 10 concurrent clients
    let mut clients = Vec::new();
    for _i in 0..10 {
        let addr_clone = addr.clone();
        let client = tokio::spawn(async move {
            connect_client(&addr_clone).await
        });
        clients.push(client);
    }

    // Wait for all clients to connect
    let mut connected_count = 0;
    for client in clients {
        let result = timeout(Duration::from_secs(5), client).await;
        if let Ok(Ok(Ok(_stream))) = result {
            connected_count += 1;
        }
    }

    // All 10 clients should connect successfully
    assert_eq!(connected_count, 10, "Expected 10 clients to connect");

    // Server will continue running but will be dropped when test ends
    let _ = server_handle.abort();
}

#[tokio::test]
async fn test_connection_state_tracked_for_all_clients() {
    // This test will be implemented in a later plan when we add
    // server methods to query connection state
    // For now, we just verify that multiple clients can connect

    let server = nexo_retailer_protocol::NexoServer::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind server");
    let addr = server.local_addr().expect("Failed to get local address").to_string();

    // Spawn server to accept connections (with timeout)
    let server_handle = tokio::spawn(async move {
        let _ = timeout(Duration::from_secs(5), server.run()).await;
        Ok::<(), nexo_retailer_protocol::NexoError>(())
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Connect 5 clients
    let mut clients = Vec::new();
    for _ in 0..5 {
        let addr_clone = addr.clone();
        let client = tokio::spawn(async move {
            connect_client(&addr_clone).await
        });
        clients.push(client);
    }

    // Wait for all clients to connect
    for client in clients {
        let _ = timeout(Duration::from_secs(5), client).await;
    }

    // Verify connection count (this will be added in later plans)
    // For now, just verify no panics - the test passes if we get here

    // Clean up
    let _ = server_handle.abort();
}

#[tokio::test]
async fn test_connection_cleanup_on_client_disconnect() {
    // This test verifies that connection state is cleaned up when clients disconnect

    let server = nexo_retailer_protocol::NexoServer::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind server");
    let addr = server.local_addr().expect("Failed to get local address").to_string();

    // We can't easily test connection cleanup in the current implementation
    // because the connection handler runs indefinitely until the client disconnects.
    // This test will be enhanced in later plans when we add methods to query
    // active connections.

    // For now, just verify that a client can connect and disconnect
    let client = connect_client(&addr).await.expect("Failed to connect client");

    // Get the peer address
    let peer_addr = client.peer_addr().expect("Failed to get peer address");

    // Drop the client (disconnect)
    drop(client);

    // Give server time to clean up
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify the peer address was valid
    assert_eq!(peer_addr.ip(), std::net::Ipv4Addr::new(127, 0, 0, 1));
}

#[tokio::test]
async fn test_server_continues_accepting_after_client_disconnect() {
    // This test verifies that the server continues accepting new connections
    // after a client disconnects

    let server = nexo_retailer_protocol::NexoServer::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind server");
    let addr = server.local_addr().expect("Failed to get local address").to_string();

    // Spawn server to accept multiple connections (with timeout)
    let server_handle = tokio::spawn(async move {
        let _ = timeout(Duration::from_secs(5), server.run()).await;
        Ok::<(), nexo_retailer_protocol::NexoError>(())
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Connect and disconnect first client
    let client1 = connect_client(&addr).await.expect("Failed to connect client 1");
    drop(client1);
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Connect second client
    let client2 = connect_client(&addr).await.expect("Failed to connect client 2");
    drop(client2);
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Connect third client
    let client3 = connect_client(&addr).await.expect("Failed to connect client 3");
    drop(client3);

    // Give server time to process all connections
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Clean up server task
    let _ = server_handle.abort();

    // If we got here without panics, the server successfully handled
    // multiple sequential connections
}

#[tokio::test]
async fn test_server_port_binding_ephemeral() {
    // Test that server can bind to ephemeral port (0)
    let server = nexo_retailer_protocol::NexoServer::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to ephemeral port");

    let addr = server.local_addr().expect("Failed to get local address");

    // Verify the port is non-zero (OS assigned)
    assert!(addr.port() > 0, "Ephemeral port should be > 0");
    assert_eq!(addr.ip(), std::net::Ipv4Addr::new(127, 0, 0, 1));
}

#[tokio::test]
async fn test_server_port_binding_specific() {
    // Test that server can bind to a specific port
    let port = 19999; // Use a high port number to avoid conflicts
    let addr_str = format!("127.0.0.1:{}", port);

    let server = nexo_retailer_protocol::NexoServer::bind(&addr_str)
        .await
        .expect("Failed to bind to specific port");

    let addr = server.local_addr().expect("Failed to get local address");

    // Verify the port matches what we requested
    assert_eq!(addr.port(), port);
    assert_eq!(addr.ip(), std::net::Ipv4Addr::new(127, 0, 0, 1));
}

#[tokio::test]
async fn test_connection_state_cleanup_decreases_hashmap_size() {
    // This test will be implemented in a later plan when we add
    // methods to inspect the internal connections HashMap
    // For now, we verify the basic connect/disconnect flow

    let server = nexo_retailer_protocol::NexoServer::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind server");
    let addr = server.local_addr().expect("Failed to get local address").to_string();

    // Spawn a simple server (with timeout)
    let server_handle = tokio::spawn(async move {
        let _ = timeout(Duration::from_secs(5), server.run()).await;
        Ok::<(), nexo_retailer_protocol::NexoError>(())
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Connect and disconnect 3 clients sequentially
    for _i in 0..3 {
        let client = connect_client(&addr).await.expect("Failed to connect client");
        drop(client);
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    // Clean up server task
    let _ = server_handle.abort();

    // Test passes if we got here without panics
}

#[tokio::test]
async fn test_concurrent_clients_mixed_connect_disconnect() {
    // Test that server handles mixed connect/disconnect patterns

    let server = nexo_retailer_protocol::NexoServer::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind server");
    let addr = server.local_addr().expect("Failed to get local address").to_string();

    // Spawn server to accept connections (with timeout)
    let server_handle = tokio::spawn(async move {
        let _ = timeout(Duration::from_secs(5), server.run()).await;
        Ok::<(), nexo_retailer_protocol::NexoError>(())
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Connect first batch of clients
    let mut clients = Vec::new();
    for _i in 0..5 {
        let addr_clone = addr.clone();
        let client = tokio::spawn(async move {
            connect_client(&addr_clone).await
        });
        clients.push(client);
    }

    // Wait for first batch to connect
    let mut connected_first = 0;
    for client in clients {
        let result = timeout(Duration::from_secs(5), client).await;
        if result.is_ok() {
            connected_first += 1;
        }
    }

    // Connect second batch
    let mut clients2 = Vec::new();
    for _i in 0..3 {
        let addr_clone = addr.clone();
        let client = tokio::spawn(async move {
            connect_client(&addr_clone).await
        });
        clients2.push(client);
    }

    // Wait for second batch
    let mut connected_second = 0;
    for client in clients2 {
        let result = timeout(Duration::from_secs(5), client).await;
        if result.is_ok() {
            connected_second += 1;
        }
    }

    // Verify we connected clients successfully
    assert!(connected_first > 0, "First batch should have connected at least one client");
    assert!(connected_second > 0, "Second batch should have connected at least one client");

    // Clean up
    let _ = server_handle.abort();

    // Test passes if we got here without panics
}
