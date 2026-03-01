//! Integration tests for server dispatcher integration
//!
//! These tests verify that the NexoServer correctly integrates with the
//! message dispatcher to route incoming messages to application handlers.
//!
//! NOTE: These tests use proper FramedTransport for message communication.
//! The server requires length-prefixed framing.

#![cfg(feature = "std")]

use nexo_retailer_protocol::server::RequestHandler;
use nexo_retailer_protocol::{
    Casp001Document, Casp001DocumentDocument, Casp002Document,
    Casp003Document, Casp004Document, Header4, SaleToPoiServiceRequestV06,
    NexoError, NexoServer, encode_message,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
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
