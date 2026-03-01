//! Integration tests for Tokio transport timeout behavior
//!
//! These tests verify that the Tokio transport properly enforces timeout limits
//! at connection, read, and write levels in realistic scenarios.

#![cfg(feature = "std")]

use core::time::Duration;
use nexo_retailer_protocol::error::NexoError;
use nexo_retailer_protocol::transport::{Transport, TokioTransport};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::time::sleep;

/// Test connection timeout to unreachable host
///
/// Uses TEST-NET-1 (192.0.2.0/24) which is reserved for documentation
/// and should never be reachable, ensuring the test is hermetic.
#[tokio::test]
async fn test_connect_timeout_unreachable_host() {
    let transport = TokioTransport::connect("192.0.2.1:8080", Duration::from_millis(100)).await;

    match transport {
        Err(NexoError::Timeout { .. }) => {
            // Expected: connection timeout
        }
        Err(other) => {
            panic!("Expected Timeout error, got: {:?}", other);
        }
        Ok(_) => {
            panic!("Expected connection to timeout");
        }
    }
}

/// Test connection timeout when port is filtered
///
/// Connects to localhost with a very short timeout. The connection
/// should timeout if no server is listening.
#[tokio::test]
async fn test_connect_timeout_port_filtered() {
    // Bind to port 0 to get a random unused port
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let port = addr.port();

    // Drop the listener so the port becomes "filtered" (no server listening)
    drop(listener);

    // Try to connect with a short timeout - should timeout or get connection refused
    let result = TokioTransport::connect(
        &format!("127.0.0.1:{}", port),
        Duration::from_millis(100),
    )
    .await;

    match result {
        Err(NexoError::Timeout { .. }) | Err(NexoError::Connection { .. }) => {
            // Either timeout or connection refused is acceptable
            // (depends on OS network stack behavior)
        }
        Err(other) => {
            panic!("Expected Timeout or Connection error, got: {:?}", other);
        }
        Ok(_) => {
            panic!("Expected connection to fail");
        }
    }
}

/// Test read timeout when server accepts connection but never sends data
///
/// Server accepts the connection but never writes any data, causing
/// the client's read operation to timeout.
#[tokio::test]
async fn test_read_timeout_server_silent() {
    // Start a server that accepts but never sends
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();

    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        // Keep connection open but never send data
        sleep(Duration::from_secs(10)).await;
        let _ = socket.shutdown().await;
    });

    // Connect with short read timeout
    let mut transport =
        TokioTransport::connect(&addr, Duration::from_secs(1))
            .await
            .unwrap();

    // Configure short read timeout
    transport = transport.with_timeouts(Duration::from_secs(1), Duration::from_millis(100));

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

/// Test read timeout when server sends data slower than timeout
///
/// Server sends data with delays between bytes, causing the read
/// operation to timeout before completing.
#[tokio::test]
async fn test_read_timeout_slow_server() {
    // Start a server that sends slowly
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();

    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();

        // Send data very slowly (200ms per byte)
        for i in 0..10u8 {
            socket.write_all(&[i]).await.unwrap();
            sleep(Duration::from_millis(200)).await;
        }

        let _ = socket.shutdown().await;
    });

    // Connect with short read timeout
    let mut transport =
        TokioTransport::connect(&addr, Duration::from_secs(1))
            .await
            .unwrap();

    // Configure read timeout shorter than server send rate
    transport = transport.with_timeouts(Duration::from_secs(1), Duration::from_millis(50));

    // Try to read - should timeout before getting all data
    let mut buf = [0u8; 10];
    let result = transport.read(&mut buf).await;

    match result {
        Err(NexoError::Timeout { .. }) => {
            // Expected: read timeout
        }
        Err(other) => {
            panic!("Expected Timeout error, got: {:?}", other);
        }
        Ok(n) if n < 10 => {
            // Also acceptable: got partial data before timeout
            // (Some OS might return partial reads)
        }
        Ok(_) => {
            panic!("Expected read to timeout or get partial data");
        }
    }
}

/// Test write timeout when server never reads
///
/// Server accepts connection but never reads from the socket,
/// causing the client's write buffer to fill up and eventually timeout.
#[tokio::test]
async fn test_write_timeout_buffer_full() {
    // Start a server that accepts but never reads
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();

    tokio::spawn(async move {
        let (_socket, _) = listener.accept().await.unwrap();
        // Never read from the socket
        sleep(Duration::from_secs(10)).await;
    });

    // Connect with short write timeout
    let mut transport =
        TokioTransport::connect(&addr, Duration::from_secs(1))
            .await
            .unwrap();

    // Configure short write timeout
    transport = transport.with_timeouts(Duration::from_secs(1), Duration::from_millis(100));

    // Write until buffer fills up and timeout occurs
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
                    panic!("Expected write to timeout after {} writes", max_writes);
                }
            }
        }
    }
}

/// Test that timeout doesn't affect reconnection
///
/// Verifies that after a timeout occurs, the transport can still
/// establish new connections successfully.
#[tokio::test]
async fn test_timeout_doesnt_affect_other_operations() {
    // First, cause a timeout
    let listener1 = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr1 = listener1.local_addr().unwrap().to_string();

    tokio::spawn(async move {
        let (mut _socket, _) = listener1.accept().await.unwrap();
        sleep(Duration::from_secs(10)).await;
    });

    let mut transport =
        TokioTransport::connect(&addr1, Duration::from_secs(1))
            .await
            .unwrap();
    transport = transport.with_timeouts(Duration::from_secs(1), Duration::from_millis(100));

    // This should timeout
    let mut buf = [0u8; 1024];
    let timeout_result = transport.read(&mut buf).await;
    assert!(matches!(timeout_result, Err(NexoError::Timeout { .. })));

    // Now create a working server
    let listener2 = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr2 = listener2.local_addr().unwrap().to_string();

    tokio::spawn(async move {
        let (mut socket, _) = listener2.accept().await.unwrap();
        socket.write_all(b"Hello").await.unwrap();
        let _ = socket.shutdown().await;
    });

    // Connect to the working server - should succeed
    let mut transport2 =
        TokioTransport::connect(&addr2, Duration::from_secs(1))
            .await
            .unwrap();
    transport2 = transport2.with_timeouts(Duration::from_secs(1), Duration::from_secs(1));

    let mut buf = [0u8; 5];
    let result = transport2.read(&mut buf).await;

    match result {
        Ok(5) => {
            assert_eq!(&buf, b"Hello");
        }
        other => {
            panic!("Expected successful read after timeout, got: {:?}", other);
        }
    }
}
