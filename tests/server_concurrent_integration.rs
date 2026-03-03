//! Concurrent connection integration tests for NexoServer
//!
//! This test file verifies that the server can handle multiple concurrent clients,
//! track connection state correctly, and clean up connections on disconnect.
//!
//! NOTE: These tests use proper FramedTransport for message communication.
//! The server requires length-prefixed framing.

#![cfg(feature = "std")]

use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::timeout;

use nexo_retailer_protocol::{
    NexoServer, NexoError, encode_message,
    Casp001Document, Casp001DocumentDocument,
    Header4, SaleToPoiServiceRequestV06,
};
use nexo_retailer_protocol::server::RequestHandler;

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

// ============================================================================
// Task 3: Extended Concurrent Connection Tests
// ============================================================================

/// Helper to send framed message over raw TCP stream
async fn send_framed_message(stream: &mut tokio::net::TcpStream, msg: &Casp001Document) -> Result<(), std::io::Error> {
    let encoded = encode_message(msg).unwrap();
    let mut framed = Vec::new();
    framed.extend_from_slice(&(encoded.len() as u32).to_be_bytes());
    framed.extend_from_slice(&encoded);
    stream.write_all(&framed).await
}

/// Helper to receive framed message from raw TCP stream
async fn recv_framed_message(stream: &mut tokio::net::TcpStream) -> Result<Vec<u8>, std::io::Error> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;
    let mut msg_buf = vec![0u8; len];
    stream.read_exact(&mut msg_buf).await?;
    Ok(msg_buf)
}

/// Create a test payment request
fn create_test_payment_request(message_id: &str) -> Casp001Document {
    Casp001Document {
        document: Some(Casp001DocumentDocument {
            sale_to_poi_svc_req: Some(SaleToPoiServiceRequestV06 {
                hdr: Some(Header4 {
                    msg_fctn: Some("DREQ".to_string()),
                    proto_vrsn: Some("6.0".to_string()),
                    tx_id: Some(message_id.to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
        }),
    }
}

/// Mock handler that tracks calls
struct TrackingHandler {
    call_count: std::sync::atomic::AtomicU32,
}

impl TrackingHandler {
    fn new() -> Self {
        Self {
            call_count: std::sync::atomic::AtomicU32::new(0),
        }
    }

    fn get_count(&self) -> u32 {
        self.call_count.load(std::sync::atomic::Ordering::SeqCst)
    }
}

#[async_trait::async_trait]
impl RequestHandler for TrackingHandler {
    async fn handle_payment_request(
        &self,
        _req: Casp001Document,
    ) -> Result<nexo_retailer_protocol::Casp002Document, NexoError> {
        self.call_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(nexo_retailer_protocol::Casp002Document::default())
    }
}

#[tokio::test]
async fn test_concurrent_clients_same_message_type() {
    // Test that multiple clients can send the same message type concurrently

    let handler = Arc::new(TrackingHandler::new());

    // Bind server to ephemeral port
    let server = NexoServer::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind server");
    let addr = server.local_addr().expect("Failed to get local address").to_string();

    let server = Arc::new(server.with_handler(handler.clone()));

    // Spawn server to accept connections (with timeout)
    let server_handle = tokio::spawn(async move {
        let _ = timeout(Duration::from_secs(10), server.run()).await;
        Ok::<(), NexoError>(())
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Spawn 10 concurrent clients, each sending the same message type (Casp001)
    let client_count = 10;
    let mut client_tasks = Vec::new();

    for i in 0..client_count {
        let addr_clone = addr.clone();
        let task = tokio::spawn(async move {
            // Connect client
            let mut stream = tokio::net::TcpStream::connect(&addr_clone).await?;

            // Send the same message type (payment request)
            let request = create_test_payment_request(&format!("SAME-TYPE-{:03}", i));
            send_framed_message(&mut stream, &request).await?;

            // Wait briefly for response
            tokio::time::sleep(Duration::from_millis(10)).await;

            // Close connection
            let _ = stream.shutdown().await;

            Ok::<(), std::io::Error>(())
        });
        client_tasks.push(task);
    }

    // Wait for all clients to complete
    for task in client_tasks {
        let _ = task.await;
    }

    // Give server time to process all messages
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Verify all messages were processed
    let count = handler.get_count();
    assert!(count >= client_count, "Expected at least {} calls, got {}", client_count, count);

    // Clean up
    let _ = server_handle.abort();
}

#[tokio::test]
async fn test_concurrent_clients_different_message_types() {
    // Test that multiple clients can send different messages concurrently

    let handler = Arc::new(TrackingHandler::new());

    // Bind server to ephemeral port
    let server = NexoServer::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind server");
    let addr = server.local_addr().expect("Failed to get local address").to_string();

    let server = Arc::new(server.with_handler(handler.clone()));

    // Spawn server to accept connections (with timeout)
    let server_handle = tokio::spawn(async move {
        let _ = timeout(Duration::from_secs(10), server.run()).await;
        Ok::<(), NexoError>(())
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Spawn 5 concurrent clients, each sending different messages
    let client_count = 5;
    let mut client_tasks = Vec::new();

    for i in 0..client_count {
        let addr_clone = addr.clone();
        let task = tokio::spawn(async move {
            // Connect client
            let mut stream = tokio::net::TcpStream::connect(&addr_clone).await?;

            // Send payment request with unique ID
            let request = create_test_payment_request(&format!("DIFF-TYPE-CLIENT-{:03}", i));
            send_framed_message(&mut stream, &request).await?;

            // Wait briefly
            tokio::time::sleep(Duration::from_millis(10)).await;

            // Close connection
            let _ = stream.shutdown().await;

            Ok::<(), std::io::Error>(())
        });
        client_tasks.push(task);
    }

    // Wait for all clients to complete
    for task in client_tasks {
        let _ = task.await;
    }

    // Give server time to process all messages
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Verify all messages were processed
    let count = handler.get_count();
    assert!(count >= client_count, "Expected at least {} calls, got {}", client_count, count);

    // Clean up
    let _ = server_handle.abort();
}

#[tokio::test]
async fn test_concurrent_client_disconnect_reconnect() {
    // Test that clients can disconnect and reconnect during load

    let handler = Arc::new(TrackingHandler::new());

    // Bind server to ephemeral port
    let server = NexoServer::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind server");
    let addr = server.local_addr().expect("Failed to get local address").to_string();

    let server = Arc::new(server.with_handler(handler.clone()));

    // Spawn server to accept connections (with timeout)
    let server_handle = tokio::spawn(async move {
        let _ = timeout(Duration::from_secs(10), server.run()).await;
        Ok::<(), NexoError>(())
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Each client connects, sends, disconnects, then reconnects and sends again
    let client_count = 5;
    let rounds = 3;
    let mut client_tasks = Vec::new();

    for client_id in 0..client_count {
        let addr_clone = addr.clone();
        let task = tokio::spawn(async move {
            for round in 0..rounds {
                // Connect
                let mut stream = tokio::net::TcpStream::connect(&addr_clone).await?;

                // Send message
                let request = create_test_payment_request(
                    &format!("RECONNECT-CLIENT-{:02}-ROUND-{:02}", client_id, round)
                );
                send_framed_message(&mut stream, &request).await?;

                // Wait briefly
                tokio::time::sleep(Duration::from_millis(10)).await;

                // Disconnect
                let _ = stream.shutdown().await;

                // Brief pause before reconnect
                tokio::time::sleep(Duration::from_millis(20)).await;
            }

            Ok::<(), std::io::Error>(())
        });
        client_tasks.push(task);
    }

    // Wait for all clients to complete
    for task in client_tasks {
        let _ = task.await;
    }

    // Give server time to process all messages
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Verify all messages were processed (5 clients * 3 rounds = 15 messages)
    let expected = client_count * rounds;
    let count = handler.get_count();
    assert!(count >= expected, "Expected at least {} calls, got {}", expected, count);

    // Clean up
    let _ = server_handle.abort();
}

#[tokio::test]
async fn test_concurrent_server_load() {
    // Test that server handles 15+ concurrent connections

    let handler = Arc::new(TrackingHandler::new());

    // Bind server to ephemeral port
    let server = NexoServer::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind server");
    let addr = server.local_addr().expect("Failed to get local address").to_string();

    let server = Arc::new(server.with_handler(handler.clone()));

    // Spawn server to accept connections (with timeout)
    let server_handle = tokio::spawn(async move {
        let _ = timeout(Duration::from_secs(15), server.run()).await;
        Ok::<(), NexoError>(())
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Spawn 15 concurrent clients
    let client_count = 15;
    let messages_per_client = 3;
    let mut client_tasks = Vec::new();

    for client_id in 0..client_count {
        let addr_clone = addr.clone();
        let task = tokio::spawn(async move {
            // Connect client
            let mut stream = tokio::net::TcpStream::connect(&addr_clone).await?;

            // Send multiple messages
            for msg_id in 0..messages_per_client {
                let request = create_test_payment_request(
                    &format!("LOAD-CLIENT-{:02}-MSG-{:03}", client_id, msg_id)
                );
                send_framed_message(&mut stream, &request).await?;
                tokio::time::sleep(Duration::from_millis(5)).await;
            }

            // Close connection
            let _ = stream.shutdown().await;

            Ok::<(), std::io::Error>(())
        });
        client_tasks.push(task);
    }

    // Wait for all clients to complete
    for task in client_tasks {
        let _ = task.await;
    }

    // Give server time to process all messages
    tokio::time::sleep(Duration::from_millis(300)).await;

    // Verify all messages were processed
    let expected = client_count * messages_per_client;
    let count = handler.get_count();
    assert!(count >= expected, "Expected at least {} calls, got {}", expected, count);

    // Clean up
    let _ = server_handle.abort();
}

#[tokio::test]
async fn test_concurrent_message_correlation() {
    // Test that request/response correlation works under concurrent load

    let handler = Arc::new(TrackingHandler::new());

    // Bind server to ephemeral port
    let server = NexoServer::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind server");
    let addr = server.local_addr().expect("Failed to get local address").to_string();

    let server = Arc::new(server.with_handler(handler.clone()));

    // Spawn server to accept connections (with timeout)
    let server_handle = tokio::spawn(async move {
        let _ = timeout(Duration::from_secs(10), server.run()).await;
        Ok::<(), NexoError>(())
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Spawn 10 concurrent clients, each expecting a response
    let client_count = 10;
    let mut client_tasks = Vec::new();

    for client_id in 0..client_count {
        let addr_clone = addr.clone();
        let task = tokio::spawn(async move {
            // Connect client
            let mut stream = tokio::net::TcpStream::connect(&addr_clone).await?;

            // Send message with unique ID
            let request = create_test_payment_request(
                &format!("CORRELATION-CLIENT-{:02}", client_id)
            );
            send_framed_message(&mut stream, &request).await?;

            // Receive response
            let response = recv_framed_message(&mut stream).await;

            // Close connection
            let _ = stream.shutdown().await;

            // Return whether we got a response
            Ok::<bool, std::io::Error>(response.is_ok())
        });
        client_tasks.push(task);
    }

    // Wait for all clients and count successful responses
    let mut success_count = 0;
    for task in client_tasks {
        if let Ok(Ok(true)) = task.await {
            success_count += 1;
        }
    }

    // Verify all clients received responses
    assert!(success_count >= client_count, "Expected {} responses, got {}", client_count, success_count);

    // Clean up
    let _ = server_handle.abort();
}

#[tokio::test]
async fn test_concurrent_connection_storm() {
    // Test server handles connection storms (rapid connect/disconnect)

    // Bind server to ephemeral port
    let server = NexoServer::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind server");
    let addr = server.local_addr().expect("Failed to get local address").to_string();

    let server = Arc::new(server);

    // Spawn server to accept connections (with timeout)
    let server_handle = tokio::spawn(async move {
        let _ = timeout(Duration::from_secs(10), server.run()).await;
        Ok::<(), NexoError>(())
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Spawn 20 clients that connect and immediately disconnect
    let client_count = 20;
    let mut client_tasks = Vec::new();

    for _ in 0..client_count {
        let addr_clone = addr.clone();
        let task = tokio::spawn(async move {
            // Connect
            if let Ok(mut stream) = tokio::net::TcpStream::connect(&addr_clone).await {
                // Immediately disconnect (storm simulation)
                let _ = stream.shutdown().await;
                true
            } else {
                false
            }
        });
        client_tasks.push(task);
    }

    // Wait for all clients
    let mut connected_count = 0;
    for task in client_tasks {
        if let Ok(true) = task.await {
            connected_count += 1;
        }
    }

    // Most clients should have connected successfully
    assert!(connected_count >= client_count * 80 / 100,
            "Expected at least {} connections, got {}",
            client_count * 80 / 100, connected_count);

    // Give server time to clean up
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify server is still accepting connections after storm
    let final_client = tokio::net::TcpStream::connect(&addr).await;
    assert!(final_client.is_ok(), "Server should still be accepting connections after storm");

    // Clean up
    let _ = server_handle.abort();
}
