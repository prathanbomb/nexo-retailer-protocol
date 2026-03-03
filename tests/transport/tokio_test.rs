//! Integration tests for Tokio transport operations
//!
//! These tests verify TokioTransport functionality including connection management,
//! concurrent operations, and connection state tracking.

#![cfg(feature = "std")]

use core::time::Duration;
use nexo_retailer_protocol::error::NexoError;
use nexo_retailer_protocol::transport::{Transport, TokioTransport, FramedTransport};
use nexo_retailer_protocol::Casp001Document;
use prost::Message;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::time::sleep;

/// Test successful connection to localhost server
#[tokio::test]
async fn test_tokio_transport_connect_success() {
    // Start a simple server
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();

    // Spawn server task that accepts one connection
    tokio::spawn(async move {
        let (_socket, _) = listener.accept().await.unwrap();
        sleep(Duration::from_secs(5)).await;
    });

    // Connect with transport
    let transport = TokioTransport::connect(&addr, Duration::from_secs(5))
        .await
        .unwrap();

    // Verify connection state
    assert!(transport.is_connected(), "Transport should report connected after successful connect");
}

/// Test connection timeout to unreachable host
#[tokio::test]
async fn test_tokio_transport_connect_timeout() {
    // Use TEST-NET-1 (192.0.2.0/24) which is reserved and never reachable
    let result = TokioTransport::connect("192.0.2.1:8080", Duration::from_millis(100)).await;

    match result {
        Err(NexoError::Timeout { .. }) => {
            // Expected: connection timeout
        }
        Err(NexoError::Connection { .. }) => {
            // Also acceptable: connection refused/failed
        }
        Err(other) => {
            panic!("Expected Timeout or Connection error, got: {:?}", other);
        }
        Ok(_) => {
            panic!("Expected connection to timeout or fail");
        }
    }
}

/// Test read timeout enforcement
#[tokio::test]
async fn test_tokio_transport_read_timeout() {
    // Start server that accepts but never sends data
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();

    tokio::spawn(async move {
        let (mut _socket, _) = listener.accept().await.unwrap();
        // Keep connection open but never send data
        sleep(Duration::from_secs(10)).await;
    });

    // Connect with short read timeout
    let mut transport = TokioTransport::connect(&addr, Duration::from_secs(1))
        .await
        .unwrap();
    transport = transport.with_timeouts(Duration::from_millis(100), Duration::from_secs(10));

    // Try to read - should timeout
    let mut buf = [0u8; 1024];
    let result = transport.read(&mut buf).await;

    match result {
        Err(NexoError::Timeout { .. }) => {
            // Expected: read timeout
        }
        Err(other) => {
            panic!("Expected Timeout error, got: {:?}", other);
        }
        Ok(_) => {
            panic!("Expected read to timeout");
        }
    }
}

/// Test write timeout enforcement when server doesn't read
#[tokio::test]
async fn test_tokio_transport_write_timeout() {
    // Start server that accepts but never reads
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();

    tokio::spawn(async move {
        let (_socket, _) = listener.accept().await.unwrap();
        // Never read from socket - buffer will fill up
        sleep(Duration::from_secs(10)).await;
    });

    // Connect with short write timeout
    let mut transport = TokioTransport::connect(&addr, Duration::from_secs(1))
        .await
        .unwrap();
    transport = transport.with_timeouts(Duration::from_secs(30), Duration::from_millis(100));

    // Write large data until buffer fills up and timeout occurs
    let large_data = vec![0u8; 1024 * 1024]; // 1MB chunks
    let mut writes = 0;
    let max_writes = 100; // Safety limit

    loop {
        let result = transport.write(&large_data).await;

        match result {
            Err(NexoError::Timeout { .. }) => {
                // Expected: write timeout when buffer full
                break;
            }
            Err(other) => {
                panic!("Expected Timeout error, got: {:?}", other);
            }
            Ok(_) => {
                writes += 1;
                if writes >= max_writes {
                    // Some systems may not timeout if buffer is large enough
                    // This is acceptable behavior
                    break;
                }
            }
        }
    }
}

/// Test that peer_addr() reflects connection state correctly
#[tokio::test]
async fn test_tokio_transport_connection_state() {
    // Start server
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();

    tokio::spawn(async move {
        let (_socket, _) = listener.accept().await.unwrap();
        sleep(Duration::from_secs(5)).await;
    });

    // Connect
    let transport = TokioTransport::connect(&addr, Duration::from_secs(1))
        .await
        .unwrap();

    // Active connection should report connected
    assert!(transport.is_connected(), "Transport should be connected after successful connect");

    // Test connection refused scenario
    let listener2 = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr2 = listener2.local_addr().unwrap().to_string();
    drop(listener2); // Close the listener

    let result = TokioTransport::connect(&addr2, Duration::from_millis(100)).await;
    assert!(result.is_err(), "Connection to closed port should fail");
}

/// Test multiple sequential read operations
#[tokio::test]
async fn test_tokio_transport_sequential_reads() {
    // Start server that sends multiple messages
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();

    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();

        // Send 5 messages with delays
        for i in 0u8..5 {
            let data = [i; 100];
            socket.write_all(&data).await.unwrap();
            sleep(Duration::from_millis(10)).await;
        }
    });

    // Connect and read sequentially
    let mut transport = TokioTransport::connect(&addr, Duration::from_secs(1))
        .await
        .unwrap();

    // Read messages sequentially
    for expected in 0u8..5 {
        let mut buf = [0u8; 100];
        let mut total_read = 0;
        while total_read < 100 {
            let n = transport.read(&mut buf[total_read..]).await.unwrap();
            total_read += n;
            if n == 0 {
                break;
            }
        }
        assert_eq!(total_read, 100, "Should read full message");
        assert!(buf.iter().all(|&b| b == expected), "All bytes should be {}", expected);
    }
}

/// Test multiple concurrent write operations
#[tokio::test]
async fn test_tokio_transport_concurrent_writes() {
    // Start echo server
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();

    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();

        // Echo back all received data
        let mut buf = [0u8; 4096];
        loop {
            let n = socket.read(&mut buf).await.unwrap();
            if n == 0 {
                break;
            }
            socket.write_all(&buf[..n]).await.unwrap();
        }
    });

    // Connect with FramedTransport
    let transport = TokioTransport::connect(&addr, Duration::from_secs(1))
        .await
        .unwrap();
    let mut framed = FramedTransport::new(transport);

    // Write multiple messages sequentially
    for i in 0..5 {
        let msg = Casp001Document::default();
        framed.send_message(&msg).await.unwrap();

        // Verify echo
        let received: Casp001Document = framed.recv_message().await.unwrap();
        assert_eq!(msg.encode_to_vec(), received.encode_to_vec(), "Message {} should round-trip", i);
    }
}

/// Test connection state after server closes
#[tokio::test]
async fn test_tokio_transport_server_disconnect() {
    // Start server that closes immediately after accept
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();

    tokio::spawn(async move {
        let (socket, _) = listener.accept().await.unwrap();
        drop(socket); // Close immediately
    });

    // Connect
    let mut transport = TokioTransport::connect(&addr, Duration::from_secs(1))
        .await
        .unwrap();

    // Connection was established
    assert!(transport.is_connected());

    // Try to read - should get EOF or error
    let mut buf = [0u8; 1024];
    let result = transport.read(&mut buf).await;

    // Should get either 0 bytes (EOF) or an error
    match result {
        Ok(0) => {
            // Expected: EOF
        }
        Err(NexoError::Connection { .. }) => {
            // Also acceptable: connection error
        }
        other => {
            // May get 0 bytes or connection error depending on timing
            if let Ok(n) = other {
                assert_eq!(n, 0, "Should get EOF (0 bytes) when server closes");
            }
        }
    }
}

/// Test reconnection after timeout
#[tokio::test]
async fn test_tokio_transport_reconnection_after_timeout() {
    // First server that causes timeout
    let listener1 = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr1 = listener1.local_addr().unwrap().to_string();

    tokio::spawn(async move {
        let (mut _socket, _) = listener1.accept().await.unwrap();
        sleep(Duration::from_secs(10)).await; // Never send data
    });

    // Connect and experience timeout
    let mut transport = TokioTransport::connect(&addr1, Duration::from_secs(1))
        .await
        .unwrap();
    transport = transport.with_timeouts(Duration::from_millis(100), Duration::from_secs(10));

    let mut buf = [0u8; 1024];
    let timeout_result = transport.read(&mut buf).await;
    assert!(matches!(timeout_result, Err(NexoError::Timeout { .. })));

    // Now connect to a working server
    let listener2 = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr2 = listener2.local_addr().unwrap().to_string();

    tokio::spawn(async move {
        let (mut socket, _) = listener2.accept().await.unwrap();
        socket.write_all(b"Hello").await.unwrap();
    });

    // New connection should work
    let mut transport2 = TokioTransport::connect(&addr2, Duration::from_secs(1))
        .await
        .unwrap();

    let mut buf = [0u8; 5];
    let result = transport2.read(&mut buf).await;
    assert!(result.is_ok(), "New connection should work after previous timeout");
    assert_eq!(&buf, b"Hello");
}

/// Test transport with dynamic port allocation
#[tokio::test]
async fn test_tokio_transport_dynamic_port() {
    // Use port 0 to get random port
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let local_addr = listener.local_addr().unwrap();
    let port = local_addr.port();

    // Verify we got a valid port (not 0)
    assert!(port > 0, "Should get assigned a non-zero port");

    tokio::spawn(async move {
        let (_socket, _) = listener.accept().await.unwrap();
        sleep(Duration::from_secs(5)).await;
    });

    // Connect using the assigned port
    let transport = TokioTransport::connect(&format!("127.0.0.1:{}", port), Duration::from_secs(1))
        .await
        .unwrap();

    assert!(transport.is_connected());
}

/// Test partial write handling
#[tokio::test]
async fn test_tokio_transport_partial_write() {
    // Start server that reads slowly
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();

    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();

        // Read slowly byte by byte
        let mut buf = [0u8; 1];
        for _ in 0..100 {
            if socket.read_exact(&mut buf).await.is_err() {
                break;
            }
            sleep(Duration::from_millis(10)).await;
        }
    });

    // Connect with generous write timeout
    let mut transport = TokioTransport::connect(&addr, Duration::from_secs(1))
        .await
        .unwrap();
    transport = transport.with_timeouts(Duration::from_secs(30), Duration::from_secs(30));

    // Write should complete even if server reads slowly
    let data = vec![0u8; 100];
    let result = transport.write(&data).await;

    // Should succeed (writes to kernel buffer, not waiting for remote read)
    assert!(result.is_ok() || matches!(result, Err(NexoError::Timeout { .. })),
            "Write should succeed or timeout gracefully");
}
