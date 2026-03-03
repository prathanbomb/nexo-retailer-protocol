//! Integration tests for connection lifecycle and error recovery
//!
//! These tests verify that transport connections handle lifecycle events
//! correctly, including open/close cycles, partial I/O recovery, and
//! peer disconnection handling.

#![cfg(feature = "std")]

use core::time::Duration;
use nexo_retailer_protocol::error::NexoError;
use nexo_retailer_protocol::transport::{Transport, TokioTransport, FramedTransport};
use nexo_retailer_protocol::Casp001Document;
use prost::Message;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::time::sleep;

/// Test connection open/close cycles
///
/// Verifies that multiple open/close cycles work correctly without
/// resource leaks.
#[tokio::test]
async fn test_connection_open_close_cycle() {
    // Start persistent server
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();

    let server_handle = tokio::spawn(async move {
        loop {
            let (mut socket, _) = listener.accept().await.unwrap();

            // Echo loop
            let mut buf = [0u8; 4096];
            loop {
                let n = socket.read(&mut buf).await.unwrap();
                if n == 0 {
                    break;
                }
                socket.write_all(&buf[..n]).await.unwrap();
            }
        }
    });

    // Perform multiple connection cycles
    for i in 0..5 {
        let transport = TokioTransport::connect(&addr, Duration::from_secs(1))
            .await
            .unwrap_or_else(|e| panic!("Connection {} failed: {:?}", i, e));

        assert!(transport.is_connected(), "Connection {} should be connected", i);

        // Send some data
        let mut framed = FramedTransport::new(transport);
        let msg = Casp001Document::default();
        framed.send_message(&msg).await.unwrap();

        // Receive echo
        let _: Casp001Document = framed.recv_message().await.unwrap();

        // Connection will be dropped here (cycle complete)
    }

    // Cleanup
    server_handle.abort();
}

/// Test partial write recovery
///
/// Verifies that the transport handles partial writes correctly
/// (when the underlying socket doesn't accept all bytes at once).
#[tokio::test]
async fn test_connection_partial_write_recovery() {
    // Start server that reads slowly
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();

    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();

        // Read very slowly - 1 byte at a time
        let mut buf = [0u8; 1];
        let mut total = 0;
        while let Ok(n) = socket.read(&mut buf).await {
            if n == 0 {
                break;
            }
            total += n;
            // Echo back what we received
            socket.write_all(&buf[..n]).await.unwrap();
            sleep(Duration::from_millis(5)).await;
        }
    });

    // Connect with generous timeouts
    let transport = TokioTransport::connect(&addr, Duration::from_secs(1))
        .await
        .unwrap()
        .with_timeouts(Duration::from_secs(30), Duration::from_secs(30));

    let mut framed = FramedTransport::new(transport);

    // Send message - should complete despite slow server reads
    let msg = Casp001Document::default();
    let result = framed.send_message(&msg).await;
    assert!(result.is_ok(), "Send should complete despite partial writes");
}

/// Test partial read recovery
///
/// Verifies that the transport handles partial reads correctly
/// (when the underlying socket doesn't provide all bytes at once).
#[tokio::test]
async fn test_connection_partial_read_recovery() {
    // Start server that writes slowly
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();

    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();

        // Create a message
        let msg = Casp001Document::default();
        let encoded = msg.encode_to_vec();

        // Send length prefix byte-by-byte
        let len = (encoded.len() as u32).to_be_bytes();
        for byte in &len {
            socket.write_all(&[*byte]).await.unwrap();
            sleep(Duration::from_millis(10)).await;
        }

        // Send message body byte-by-byte
        for byte in &encoded {
            socket.write_all(&[*byte]).await.unwrap();
            sleep(Duration::from_millis(10)).await;
        }
    });

    // Connect with generous timeouts
    let transport = TokioTransport::connect(&addr, Duration::from_secs(5))
        .await
        .unwrap()
        .with_timeouts(Duration::from_secs(30), Duration::from_secs(30));

    let mut framed = FramedTransport::new(transport);

    // Receive message - should complete despite slow server writes
    let result: Result<Casp001Document, _> = framed.recv_message().await;
    assert!(result.is_ok(), "Receive should complete despite partial reads");

    let received = result.unwrap();
    let expected = Casp001Document::default();
    assert_eq!(expected.encode_to_vec(), received.encode_to_vec());
}

/// Test peer disconnect handling
///
/// Verifies that the transport gracefully handles unexpected peer disconnection.
#[tokio::test]
async fn test_connection_peer_disconnect() {
    // Start server that disconnects immediately
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();

    tokio::spawn(async move {
        let (socket, _) = listener.accept().await.unwrap();
        // Immediately close connection
        drop(socket);
    });

    // Connect
    let transport = TokioTransport::connect(&addr, Duration::from_secs(1))
        .await
        .unwrap();

    assert!(transport.is_connected());

    // Try to read - should get EOF or error
    let mut transport = transport;
    let mut buf = [0u8; 1024];
    let result = transport.read(&mut buf).await;

    match result {
        Ok(0) => {
            // Expected: EOF (0 bytes read)
        }
        Err(NexoError::Connection { .. }) => {
            // Expected: connection error
        }
        Err(NexoError::Timeout { .. }) => {
            // Also acceptable: timeout
        }
        other => {
            // Some systems might return different results
            if let Ok(n) = other {
                assert_eq!(n, 0, "Should get EOF when peer disconnects");
            }
        }
    }
}

/// Test connection timeout recovery
///
/// Verifies that after a timeout, new connections can be established.
#[tokio::test]
async fn test_connection_timeout_recovery() {
    // First server that causes timeout
    let listener1 = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr1 = listener1.local_addr().unwrap().to_string();

    tokio::spawn(async move {
        let (mut _socket, _) = listener1.accept().await.unwrap();
        // Never send data - cause read timeout
        sleep(Duration::from_secs(10)).await;
    });

    // Connect and experience timeout
    let mut transport = TokioTransport::connect(&addr1, Duration::from_secs(1))
        .await
        .unwrap()
        .with_timeouts(Duration::from_millis(100), Duration::from_secs(10));

    let mut buf = [0u8; 1024];
    let timeout_result = transport.read(&mut buf).await;
    assert!(matches!(timeout_result, Err(NexoError::Timeout { .. })),
            "Should get timeout when server doesn't respond");

    // Now connect to a working server
    let listener2 = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr2 = listener2.local_addr().unwrap().to_string();

    tokio::spawn(async move {
        let (mut socket, _) = listener2.accept().await.unwrap();

        // Echo server
        let mut len_buf = [0u8; 4];
        socket.read_exact(&mut len_buf).await.unwrap();
        let len = u32::from_be_bytes(len_buf) as usize;
        let mut msg_buf = vec![0u8; len];
        socket.read_exact(&mut msg_buf).await.unwrap();
        socket.write_all(&len_buf).await.unwrap();
        socket.write_all(&msg_buf).await.unwrap();
    });

    // New connection should work
    let transport = TokioTransport::connect(&addr2, Duration::from_secs(1))
        .await
        .expect("New connection should succeed after previous timeout");

    let mut framed = FramedTransport::new(transport);
    let msg = Casp001Document::default();
    framed.send_message(&msg).await.expect("Send should succeed");

    let received: Casp001Document = framed.recv_message().await.expect("Receive should succeed");
    assert_eq!(msg.encode_to_vec(), received.encode_to_vec());
}

/// Test concurrent connection handling
///
/// Verifies that multiple connections can be handled concurrently.
#[tokio::test]
async fn test_connection_concurrent_handling() {
    // Start server that handles multiple connections
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();

    tokio::spawn(async move {
        for _ in 0..3 {
            let (mut socket, _) = listener.accept().await.unwrap();

            // Spawn handler for each connection
            tokio::spawn(async move {
                let mut len_buf = [0u8; 4];
                if socket.read_exact(&mut len_buf).await.is_err() {
                    return;
                }
                let len = u32::from_be_bytes(len_buf) as usize;
                let mut msg_buf = vec![0u8; len];
                if socket.read_exact(&mut msg_buf).await.is_err() {
                    return;
                }
                let _ = socket.write_all(&len_buf).await;
                let _ = socket.write_all(&msg_buf).await;
            });
        }
    });

    // Give server time to start
    sleep(Duration::from_millis(10)).await;

    // Create multiple concurrent connections
    let mut handles = vec![];

    for _ in 0..3 {
        let addr = addr.clone();
        let handle = tokio::spawn(async move {
            let transport = TokioTransport::connect(&addr, Duration::from_secs(1))
                .await
                .unwrap();

            let mut framed = FramedTransport::new(transport);
            let msg = Casp001Document::default();
            framed.send_message(&msg).await.unwrap();

            let received: Casp001Document = framed.recv_message().await.unwrap();
            assert_eq!(msg.encode_to_vec(), received.encode_to_vec());
        });
        handles.push(handle);
    }

    // Wait for all connections to complete
    for handle in handles {
        handle.await.unwrap();
    }
}

/// Test connection state after error
///
/// Verifies that connection state is correct after various error conditions.
#[tokio::test]
async fn test_connection_state_after_error() {
    // Test connection refused
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    drop(listener); // Close immediately

    let result = TokioTransport::connect(&addr, Duration::from_millis(100)).await;
    assert!(result.is_err(), "Should fail to connect to closed port");

    // Test successful connection state
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();

    tokio::spawn(async move {
        let (_socket, _) = listener.accept().await.unwrap();
        sleep(Duration::from_secs(5)).await;
    });

    let transport = TokioTransport::connect(&addr, Duration::from_secs(1))
        .await
        .unwrap();

    assert!(transport.is_connected(), "Should be connected after successful connect");
}

/// Test connection with various timeout configurations
///
/// Verifies that different timeout configurations work correctly.
#[tokio::test]
async fn test_connection_timeout_configurations() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();

    tokio::spawn(async move {
        let (_socket, _) = listener.accept().await.unwrap();
        sleep(Duration::from_secs(5)).await;
    });

    // Test with various timeout configurations
    let configs = [
        (Duration::from_millis(100), Duration::from_millis(100)),
        (Duration::from_secs(1), Duration::from_secs(1)),
        (Duration::from_secs(30), Duration::from_secs(10)),
    ];

    for (read_timeout, write_timeout) in configs {
        let transport = TokioTransport::connect(&addr, Duration::from_secs(1))
            .await
            .unwrap()
            .with_timeouts(read_timeout, write_timeout);

        assert!(transport.is_connected());
    }
}

/// Test graceful connection shutdown
///
/// Verifies that connections can be gracefully shut down.
#[tokio::test]
async fn test_connection_graceful_shutdown() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();

    let shutdown_detected = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let shutdown_clone = shutdown_detected.clone();

    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();

        // Wait for EOF (client shutdown)
        let mut buf = [0u8; 1];
        let n = socket.read(&mut buf).await.unwrap();
        if n == 0 {
            shutdown_clone.store(true, std::sync::atomic::Ordering::SeqCst);
        }
    });

    // Connect and then drop (simulating graceful shutdown)
    {
        let _transport = TokioTransport::connect(&addr, Duration::from_secs(1))
            .await
            .unwrap();
        // Transport dropped here
    }

    // Give server time to detect shutdown
    sleep(Duration::from_millis(100)).await;

    // Server should have detected the shutdown
    assert!(shutdown_detected.load(std::sync::atomic::Ordering::SeqCst),
            "Server should detect client shutdown");
}
