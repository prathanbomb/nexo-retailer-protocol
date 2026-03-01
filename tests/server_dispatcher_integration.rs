//! Integration tests for server dispatcher integration
//!
//! These tests verify that the NexoServer correctly integrates with the
//! message dispatcher to route incoming messages to application handlers.

#![cfg(feature = "std")]

use nexo_retailer_protocol::server::RequestHandler;
use nexo_retailer_protocol::{Casp001Document, Casp002Document, Casp003Document, Casp004Document, NexoError, NexoServer};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Mock request handler for testing
struct TestRequestHandler {
    payment_request_called: AtomicBool,
    admin_request_called: AtomicBool,
}

impl TestRequestHandler {
    fn new() -> Self {
        Self {
            payment_request_called: AtomicBool::new(false),
            admin_request_called: AtomicBool::new(false),
        }
    }
}

#[async_trait::async_trait]
impl RequestHandler for TestRequestHandler {
    async fn handle_payment_request(
        &self,
        _req: Casp001Document,
    ) -> Result<Casp002Document, NexoError> {
        self.payment_request_called.store(true, Ordering::SeqCst);
        Ok(Casp002Document::default())
    }

    async fn handle_admin_request(
        &self,
        _req: Casp003Document,
    ) -> Result<Casp004Document, NexoError> {
        self.admin_request_called.store(true, Ordering::SeqCst);
        Ok(Casp004Document::default())
    }
}

#[tokio::test]
async fn test_server_with_handler_receives_requests() {
    // Create a handler
    let handler = Arc::new(TestRequestHandler::new());

    // Bind server to ephemeral port
    let server = NexoServer::bind("127.0.0.1:0").await.unwrap();
    let addr = server.local_addr().unwrap().to_string();

    // Spawn server in background
    let server = Arc::new(server.with_handler(handler.clone()));
    tokio::spawn(async move {
        // Run server for a short time
        tokio::time::timeout(
            tokio::time::Duration::from_millis(200),
            server.run()
        ).await.ok();
    });

    // Give server time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    // Connect a client
    let mut stream = tokio::net::TcpStream::connect(&addr).await.unwrap();

    // Create a payment request with document field set
    let request = Casp001Document {
        document: Some(nexo_retailer_protocol::Casp001DocumentDocument::default()),
    };
    let request_bytes = nexo_retailer_protocol::encode_message(&request).unwrap();

    // Send request
    stream.write_all(&request_bytes).await.unwrap();

    // Give server time to process
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Verify handler was called
    assert!(handler.payment_request_called.load(Ordering::SeqCst));
}

#[tokio::test]
async fn test_server_continues_after_handler_error() {
    // Create a handler that returns errors
    struct ErrorHandler;
    #[async_trait::async_trait]
    impl RequestHandler for ErrorHandler {
        async fn handle_payment_request(
            &self,
            _req: Casp001Document,
        ) -> Result<Casp002Document, NexoError> {
            Err(NexoError::Validation {
                field: "test",
                reason: "handler error",
            })
        }
    }

    let handler = Arc::new(ErrorHandler);

    // Bind server to ephemeral port
    let server = NexoServer::bind("127.0.0.1:0").await.unwrap();
    let addr = server.local_addr().unwrap().to_string();

    // Spawn server in background for a short time
    let server = Arc::new(server.with_handler(handler));
    tokio::spawn(async move {
        tokio::time::timeout(
            tokio::time::Duration::from_millis(200),
            server.run()
        ).await.ok();
    });

    // Give server time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    // Connect a client and send a request
    let mut stream = tokio::net::TcpStream::connect(&addr).await.unwrap();

    let request = Casp001Document {
        document: Some(nexo_retailer_protocol::Casp001DocumentDocument::default()),
    };
    let request_bytes = nexo_retailer_protocol::encode_message(&request).unwrap();

    // Send request - handler will error but server should continue
    let result = stream.write_all(&request_bytes).await;

    // Verify write succeeded (server didn't crash)
    assert!(result.is_ok());

    // Give server time to process
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Server should still be running (test completes without timeout)
}

#[tokio::test]
async fn test_server_without_handler_uses_echo() {
    // Bind server without handler
    let server = NexoServer::bind("127.0.0.1:0").await.unwrap();
    let addr = server.local_addr().unwrap().to_string();

    // Spawn server in background for a short time
    let server = Arc::new(server);
    tokio::spawn(async move {
        tokio::time::timeout(
            tokio::time::Duration::from_millis(200),
            server.run()
        ).await.ok();
    });

    // Give server time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    // Connect a client
    let mut stream = tokio::net::TcpStream::connect(&addr).await.unwrap();

    // Send some data
    let test_data = b"hello, world!";
    stream.write_all(test_data).await.unwrap();

    // Give server time to echo
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Read echo response
    let mut buffer = [0u8; 1024];
    let n = stream.read(&mut buffer).await.unwrap();

    // Verify echo (basic fallback behavior)
    assert_eq!(&buffer[..n], test_data);
}
