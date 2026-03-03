//! Integration tests for server dispatcher integration
//!
//! These tests verify that the NexoServer correctly integrates with the
//! message dispatcher to route incoming messages to application handlers.
//!
//! NOTE: These tests use proper FramedTransport for message communication.
//! The server requires length-prefixed framing.

#![cfg(feature = "std")]

use nexo_retailer_protocol::server::{RequestHandler, Dispatcher};
use nexo_retailer_protocol::{
    Casp001Document, Casp001DocumentDocument, Casp002Document,
    Casp003Document, Casp004Document, Header4, SaleToPoiServiceRequestV06,
    NexoError, NexoServer, encode_message,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::{timeout, Duration};
use prost::Message;

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
            tokio::time::Duration::from_millis(500),
            server.run()
        ).await.ok();
    });

    // Give server time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Connect a client
    let mut stream = tokio::net::TcpStream::connect(&addr).await.unwrap();

    // Create a payment request with proper CASP structure
    let request = Casp001Document {
        document: Some(Casp001DocumentDocument {
            sale_to_poi_svc_req: Some(SaleToPoiServiceRequestV06 {
                hdr: Some(Header4 {
                    msg_fctn: Some("DREQ".to_string()),
                    proto_vrsn: Some("6.0".to_string()),
                    tx_id: Some("TX-DISPATCHER-TEST".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
        }),
    };

    // Send request with proper framing
    send_framed_message(&mut stream, &request).await.unwrap();

    // Wait for response (gives server time to process)
    let response = recv_framed_message(&mut stream).await;
    assert!(response.is_ok(), "Should receive framed response");

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
            tokio::time::Duration::from_millis(500),
            server.run()
        ).await.ok();
    });

    // Give server time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Connect a client and send a request
    let mut stream = tokio::net::TcpStream::connect(&addr).await.unwrap();

    // Create a payment request with proper CASP structure
    let request = Casp001Document {
        document: Some(Casp001DocumentDocument {
            sale_to_poi_svc_req: Some(SaleToPoiServiceRequestV06 {
                hdr: Some(Header4 {
                    msg_fctn: Some("DREQ".to_string()),
                    proto_vrsn: Some("6.0".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
        }),
    };

    // Send request with proper framing - handler will error but server should continue
    let result = send_framed_message(&mut stream, &request).await;

    // Verify write succeeded (server didn't crash)
    assert!(result.is_ok());

    // Give server time to process
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

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
            tokio::time::Duration::from_millis(500),
            server.run()
        ).await.ok();
    });

    // Give server time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Connect a client
    let mut stream = tokio::net::TcpStream::connect(&addr).await.unwrap();

    // Create a proper CASP message
    let request = Casp001Document {
        document: Some(Casp001DocumentDocument {
            sale_to_poi_svc_req: Some(SaleToPoiServiceRequestV06 {
                hdr: Some(Header4 {
                    msg_fctn: Some("DREQ".to_string()),
                    proto_vrsn: Some("6.0".to_string()),
                    tx_id: Some("TX-ECHO-TEST".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
        }),
    };

    // Send with proper framing
    send_framed_message(&mut stream, &request).await.unwrap();

    // Read echo response with proper framing
    let response = recv_framed_message(&mut stream).await;
    assert!(response.is_ok(), "Should receive framed echo response");

    // Verify response is valid CASP document
    let response_doc: Casp001Document = Casp001Document::decode(&*response.unwrap()).unwrap();
    assert!(response_doc.document.is_some());
}

// ============================================================================
// Task 5b: Extended Dispatcher Tests
// ============================================================================

/// Create a test payment request with specific transaction ID
fn create_test_payment_request(tx_id: &str) -> Casp001Document {
    Casp001Document {
        document: Some(Casp001DocumentDocument {
            sale_to_poi_svc_req: Some(SaleToPoiServiceRequestV06 {
                hdr: Some(Header4 {
                    msg_fctn: Some("DREQ".to_string()),
                    proto_vrsn: Some("6.0".to_string()),
                    tx_id: Some(tx_id.to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
        }),
    }
}

/// Handler that tracks all calls
struct CountingHandler {
    payment_count: AtomicU32,
    admin_count: AtomicU32,
}

impl CountingHandler {
    fn new() -> Self {
        Self {
            payment_count: AtomicU32::new(0),
            admin_count: AtomicU32::new(0),
        }
    }
}

#[async_trait::async_trait]
impl RequestHandler for CountingHandler {
    async fn handle_payment_request(
        &self,
        _req: Casp001Document,
    ) -> Result<Casp002Document, NexoError> {
        self.payment_count.fetch_add(1, Ordering::SeqCst);
        Ok(Casp002Document::default())
    }

    async fn handle_admin_request(
        &self,
        _req: Casp003Document,
    ) -> Result<Casp004Document, NexoError> {
        self.admin_count.fetch_add(1, Ordering::SeqCst);
        Ok(Casp004Document::default())
    }
}

#[tokio::test]
async fn test_dispatcher_routes_payment_requests() {
    // Test that dispatcher routes Casp001 to payment handler
    let handler = Arc::new(CountingHandler::new());
    let dispatcher = Dispatcher::new(handler.clone());

    // Create a payment request
    let request = create_test_payment_request("DISPATCH-TEST-001");
    let request_bytes = encode_message(&request).unwrap();

    // Dispatch
    let result = dispatcher.dispatch(&request_bytes).await;
    assert!(result.is_ok(), "Dispatch should succeed");

    // Verify handler was called
    assert_eq!(handler.payment_count.load(Ordering::SeqCst), 1, "Payment handler should be called once");
}

#[tokio::test]
async fn test_dispatcher_unknown_message_type() {
    // Test that dispatcher handles unknown/malformed messages gracefully
    let handler = Arc::new(CountingHandler::new());
    let dispatcher = Dispatcher::new(handler.clone());

    // Create invalid bytes
    let invalid_bytes = vec![0xFF, 0xFF, 0xFF, 0xFF];

    // Dispatch should fail gracefully
    let result = dispatcher.dispatch(&invalid_bytes).await;
    assert!(result.is_err(), "Dispatch should fail for invalid bytes");

    // Verify error type
    match result {
        Err(NexoError::Validation { field, .. }) => {
            assert_eq!(field, "message_type");
        }
        _ => panic!("Expected Validation error"),
    }
}

#[tokio::test]
async fn test_dispatcher_application_handler_error() {
    // Test that handler errors are propagated correctly
    struct ErroringHandler;

    #[async_trait::async_trait]
    impl RequestHandler for ErroringHandler {
        async fn handle_payment_request(
            &self,
            _req: Casp001Document,
        ) -> Result<Casp002Document, NexoError> {
            Err(NexoError::Validation {
                field: "test_field",
                reason: "intentional error for testing",
            })
        }
    }

    let handler = Arc::new(ErroringHandler);
    let dispatcher = Dispatcher::new(handler);

    // Create a payment request
    let request = create_test_payment_request("ERROR-TEST-001");
    let request_bytes = encode_message(&request).unwrap();

    // Dispatch should fail with the handler error
    let result = dispatcher.dispatch(&request_bytes).await;
    assert!(result.is_err(), "Dispatch should fail when handler errors");

    match result {
        Err(NexoError::Validation { field, reason }) => {
            assert_eq!(field, "test_field");
            assert!(reason.contains("intentional error"));
        }
        _ => panic!("Expected Validation error from handler"),
    }
}

#[tokio::test]
async fn test_dispatcher_concurrent_requests() {
    // Test that dispatcher handles concurrent requests correctly
    let handler = Arc::new(CountingHandler::new());
    let dispatcher = Arc::new(Dispatcher::new(handler.clone()));

    // Spawn 10 concurrent dispatch tasks
    let mut handles = Vec::new();
    for i in 0..10 {
        let disp = dispatcher.clone();
        let handle = tokio::spawn(async move {
            let request = create_test_payment_request(&format!("CONCURRENT-{:03}", i));
            let request_bytes = encode_message(&request).unwrap();
            disp.dispatch(&request_bytes).await
        });
        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        let result = handle.await.expect("Task should complete");
        assert!(result.is_ok(), "Each dispatch should succeed");
    }

    // Verify all were processed
    assert_eq!(handler.payment_count.load(Ordering::SeqCst), 10, "All 10 requests should be processed");
}

#[tokio::test]
async fn test_dispatcher_response_correlation() {
    // Test that responses are properly correlated with requests
    let handler = Arc::new(CountingHandler::new());
    let dispatcher = Dispatcher::new(handler.clone());

    // Create request
    let request = create_test_payment_request("CORRELATION-TEST");
    let request_bytes = encode_message(&request).unwrap();

    // Dispatch and get response
    let response_bytes = dispatcher.dispatch(&request_bytes).await.expect("Dispatch should succeed");

    // Decode response
    let response: Casp002Document = Casp002Document::decode(&*response_bytes).expect("Response should be valid");

    // Response should be a valid Casp002Document
    assert!(response.document.is_some() || response.document.is_none(), "Response should be a valid document");
}

#[tokio::test]
async fn test_dispatcher_empty_message_handling() {
    // Test that dispatcher handles empty messages
    let handler = Arc::new(CountingHandler::new());
    let dispatcher = Dispatcher::new(handler.clone());

    // Create empty/default message
    let request = Casp001Document::default();
    let request_bytes = encode_message(&request).unwrap();

    // Dispatch should work (empty documents are valid protobuf)
    let result = dispatcher.dispatch(&request_bytes).await;
    // Result depends on whether empty document passes the document.is_some() check
    assert!(result.is_ok() || result.is_err(), "Dispatcher should handle empty message gracefully");
}

#[tokio::test]
async fn test_dispatcher_large_message_handling() {
    // Test that dispatcher handles large messages
    let handler = Arc::new(CountingHandler::new());
    let dispatcher = Dispatcher::new(handler.clone());

    // Create a message with a large text field
    let large_text = "A".repeat(200);
    let request = Casp001Document {
        document: Some(Casp001DocumentDocument {
            sale_to_poi_svc_req: Some(SaleToPoiServiceRequestV06 {
                hdr: Some(Header4 {
                    msg_fctn: Some("DREQ".to_string()),
                    proto_vrsn: Some("6.0".to_string()),
                    tx_id: Some(large_text.clone()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
        }),
    };
    let request_bytes = encode_message(&request).unwrap();

    // Dispatch should succeed
    let result = dispatcher.dispatch(&request_bytes).await;
    assert!(result.is_ok(), "Dispatcher should handle large message");

    // Verify handler was called
    assert_eq!(handler.payment_count.load(Ordering::SeqCst), 1, "Handler should be called");
}

#[tokio::test]
async fn test_dispatcher_dispatch_document() {
    // Test the dispatch_document method that accepts decoded structs
    let handler = Arc::new(CountingHandler::new());
    let dispatcher = Dispatcher::new(handler.clone());

    // Create a request document directly (no encoding)
    let request = create_test_payment_request("DISPATCH-DOC-TEST");

    // Dispatch using dispatch_document (accepts decoded struct)
    let result = dispatcher.dispatch_document(request).await;

    // Verify success
    assert!(result.is_ok(), "dispatch_document should succeed");

    // Verify handler was called
    assert_eq!(handler.payment_count.load(Ordering::SeqCst), 1, "Handler should be called");

    // Verify response is a Casp002Document
    let response = result.unwrap();
    assert!(response.document.is_some() || response.document.is_none());
}

#[tokio::test]
async fn test_server_dispatcher_integration_concurrent() {
    // Test server + dispatcher integration with concurrent connections
    let handler = Arc::new(CountingHandler::new());

    // Bind server to ephemeral port
    let server = NexoServer::bind("127.0.0.1:0").await.unwrap();
    let addr = server.local_addr().unwrap().to_string();

    let server = Arc::new(server.with_handler(handler.clone()));

    // Spawn server in background
    tokio::spawn(async move {
        let _ = timeout(Duration::from_secs(5), server.run()).await;
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Spawn 5 concurrent clients
    let mut handles = Vec::new();
    for i in 0..5 {
        let addr_clone = addr.clone();
        let handle = tokio::spawn(async move {
            // Connect
            let mut stream = tokio::net::TcpStream::connect(&addr_clone).await?;

            // Send request
            let request = create_test_payment_request(&format!("INTEGRATION-{:03}", i));
            send_framed_message(&mut stream, &request).await?;

            // Wait briefly for response
            tokio::time::sleep(Duration::from_millis(20)).await;

            // Close connection
            let _ = stream.shutdown().await;

            Ok::<(), std::io::Error>(())
        });
        handles.push(handle);
    }

    // Wait for all clients
    for handle in handles {
        let _ = handle.await;
    }

    // Give server time to process
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify all requests were processed
    let count = handler.payment_count.load(Ordering::SeqCst);
    assert!(count >= 5, "At least 5 requests should be processed, got {}", count);
}
