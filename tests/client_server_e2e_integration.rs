//! End-to-end integration tests for Nexo Retailer Protocol client-server communication
//!
//! These tests verify the complete client-server communication flow using
//! FramedTransport for proper message framing. They confirm that the framing
//! protocol (4-byte big-endian length prefix) is correctly applied on both
//! send and receive paths.
//!
//! # Test Coverage
//!
//! - Client to server payment request flow
//! - Server to client response flow
//! - Heartbeat message framing
//! - Multiple concurrent clients
//!
//! # Audit Verification
//!
//! These tests verify that the 3 broken flows identified in the v1.0 milestone audit
//! now work end-to-end:
//! 1. Client -> Server Payment Request
//! 2. Server -> Client Response
//! 3. Server Heartbeat

#![cfg(feature = "std")]

use nexo_retailer_protocol::server::RequestHandler;
use nexo_retailer_protocol::transport::{FramedTransport, TokioTransport};
use nexo_retailer_protocol::{
    Casp001Document, Casp001DocumentDocument, Casp002Document,
    Header4, SaleToPoiServiceRequestV06, NexoError,
    NexoClient, NexoServer,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::time::{timeout, Duration};

// ============================================================================
// Test Handler Implementation
// ============================================================================

/// Simple handler that echoes back responses for testing
struct TestPaymentHandler {
    request_count: AtomicUsize,
}

impl TestPaymentHandler {
    fn new() -> Self {
        Self {
            request_count: AtomicUsize::new(0),
        }
    }

    fn request_count(&self) -> usize {
        self.request_count.load(Ordering::SeqCst)
    }
}

#[async_trait::async_trait]
impl RequestHandler for TestPaymentHandler {
    async fn handle_payment_request(
        &self,
        req: Casp001Document,
    ) -> Result<Casp002Document, NexoError> {
        self.request_count.fetch_add(1, Ordering::SeqCst);

        // Create a response echoing back the header
        let header = req.document.as_ref()
            .and_then(|d| d.sale_to_poi_svc_req.as_ref())
            .and_then(|r| r.hdr.as_ref());

        Ok(Casp002Document {
            document: Some(nexo_retailer_protocol::Casp002DocumentDocument {
                sale_to_poi_svc_rsp: Some(nexo_retailer_protocol::SaleToPoiServiceResponseV06 {
                    hdr: header.cloned(),
                    ..Default::default()
                }),
            }),
        })
    }
}

// ============================================================================
// E2E Test: Client to Server Payment Request Flow
// ============================================================================

#[tokio::test]
async fn test_e2e_client_server_payment_request() {
    // Create server with handler
    let handler = Arc::new(TestPaymentHandler::new());
    let server = NexoServer::bind("127.0.0.1:0").await.expect("Server bind should succeed");
    let server_addr = server.local_addr().expect("Should get server address");

    // Get handler reference for verification
    let handler_clone = Arc::clone(&handler);

    // Start server in background
    tokio::spawn(async move {
        server.with_handler(handler).run().await.expect("Server run should succeed");
    });

    // Wait for server to be ready
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Create NexoClient and connect
    let mut client = NexoClient::new();
    client.connect(&server_addr.to_string())
        .await
        .expect("Client connect should succeed");

    // Create a payment request
    let request = Casp001Document {
        document: Some(Casp001DocumentDocument {
            sale_to_poi_svc_req: Some(SaleToPoiServiceRequestV06 {
                hdr: Some(Header4 {
                    msg_fctn: Some("DREQ".to_string()),
                    proto_vrsn: Some("6.0".to_string()),
                    tx_id: Some("TX-E2E-001".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
        }),
    };

    // Send request and receive response (use separate calls for different types)
    client.send_request(&request)
        .await
        .expect("Send should succeed");
    let response: Casp002Document = client.receive_response()
        .await
        .expect("Receive should succeed");

    // Verify response was received with correct header
    assert!(response.document.is_some());
    let doc = response.document.unwrap();
    assert!(doc.sale_to_poi_svc_rsp.is_some());
    let svc_rsp = doc.sale_to_poi_svc_rsp.unwrap();
    assert!(svc_rsp.hdr.is_some());
    let hdr = svc_rsp.hdr.unwrap();
    assert_eq!(hdr.msg_fctn, Some("DREQ".to_string()));
    assert_eq!(hdr.tx_id, Some("TX-E2E-001".to_string()));

    // Verify handler was called
    assert_eq!(handler_clone.request_count(), 1);

    // Disconnect
    client.disconnect().await.expect("Disconnect should succeed");
}

// ============================================================================
// E2E Test: Server to Client Response Flow
// ============================================================================

#[tokio::test]
async fn test_e2e_server_to_client_response_flow() {
    // Create server with handler
    let handler = Arc::new(TestPaymentHandler::new());
    let server = NexoServer::bind("127.0.0.1:0").await.expect("Server bind should succeed");
    let server_addr = server.local_addr().expect("Should get server address");

    // Start server in background
    tokio::spawn(async move {
        server.with_handler(handler).run().await.expect("Server run should succeed");
    });

    // Wait for server to be ready
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Create NexoClient and connect
    let mut client = NexoClient::new();
    client.connect(&server_addr.to_string())
        .await
        .expect("Client connect should succeed");

    // Create multiple payment requests and verify responses
    for i in 0..3 {
        let request = Casp001Document {
            document: Some(Casp001DocumentDocument {
                sale_to_poi_svc_req: Some(SaleToPoiServiceRequestV06 {
                    hdr: Some(Header4 {
                        msg_fctn: Some("DREQ".to_string()),
                        proto_vrsn: Some("6.0".to_string()),
                        tx_id: Some(format!("TX-ITER-{:03}", i)),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
            }),
        };

        // Send request and receive response
        client.send_request(&request)
            .await
            .expect(&format!("Request {} send should succeed", i));
        let response: Casp002Document = client.receive_response()
            .await
            .expect(&format!("Request {} receive should succeed", i));

        // Verify response
        assert!(response.document.is_some());
        let doc = response.document.unwrap();
        assert!(doc.sale_to_poi_svc_rsp.is_some());
    }

    // Disconnect
    client.disconnect().await.expect("Disconnect should succeed");
}

// ============================================================================
// E2E Test: Heartbeat Messages with FramedTransport
// ============================================================================

#[tokio::test]
async fn test_e2e_heartbeat_with_framed_transport() {
    // Create server without handler (uses echo mode)
    let server = NexoServer::bind("127.0.0.1:0").await.expect("Server bind should succeed");
    let server_addr = server.local_addr().expect("Should get server address");

    // Start server in background (no handler - echo mode)
    tokio::spawn(async move {
        server.run().await.expect("Server run should succeed");
    });

    // Wait for server to be ready
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Connect with raw TcpStream and wrap in FramedTransport
    let stream = tokio::net::TcpStream::connect(&server_addr.to_string())
        .await
        .expect("Should connect to server");
    let transport = TokioTransport::new(stream);
    let mut framed = FramedTransport::new(transport);

    // Create a heartbeat message
    let heartbeat = Casp001Document {
        document: Some(Casp001DocumentDocument {
            sale_to_poi_svc_req: Some(SaleToPoiServiceRequestV06 {
                hdr: Some(Header4 {
                    msg_fctn: Some("HRTB".to_string()),
                    proto_vrsn: Some("6.0".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
        }),
    };

    // Send heartbeat with proper framing
    framed.send_message(&heartbeat)
        .await
        .expect("Heartbeat send should succeed");

    // Receive echoed heartbeat with proper framing
    let result = timeout(Duration::from_secs(5), framed.recv_message::<Casp001Document>()).await;

    match result {
        Ok(Ok(received)) => {
            // Verify heartbeat was echoed back
            assert!(received.document.is_some());
            let doc = received.document.unwrap();
            assert!(doc.sale_to_poi_svc_req.is_some());
            let svc_req = doc.sale_to_poi_svc_req.unwrap();
            assert!(svc_req.hdr.is_some());
            let hdr = svc_req.hdr.unwrap();
            assert_eq!(hdr.msg_fctn, Some("HRTB".to_string()));
        }
        Ok(Err(e)) => {
            panic!("Failed to receive heartbeat: {:?}", e);
        }
        Err(_) => {
            panic!("Timeout waiting for heartbeat response");
        }
    }
}

// ============================================================================
// E2E Test: Multiple Concurrent Clients
// ============================================================================

#[tokio::test]
async fn test_e2e_concurrent_clients() {
    // Create server with handler
    let handler = Arc::new(TestPaymentHandler::new());
    let server = NexoServer::bind("127.0.0.1:0").await.expect("Server bind should succeed");
    let server_addr = server.local_addr().expect("Should get server address");

    // Start server in background
    tokio::spawn(async move {
        server.with_handler(handler).run().await.expect("Server run should succeed");
    });

    // Wait for server to be ready
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Spawn 5 concurrent clients
    let mut tasks = Vec::new();
    let num_clients = 5;

    for client_id in 0..num_clients {
        let addr = server_addr.to_string();
        let task = tokio::spawn(async move {
            // Create client and connect
            let mut client = NexoClient::new();
            client.connect(&addr)
                .await
                .expect(&format!("Client {} connect should succeed", client_id));

            // Send a payment request
            let request = Casp001Document {
                document: Some(Casp001DocumentDocument {
                    sale_to_poi_svc_req: Some(SaleToPoiServiceRequestV06 {
                        hdr: Some(Header4 {
                            msg_fctn: Some("DREQ".to_string()),
                            proto_vrsn: Some("6.0".to_string()),
                            tx_id: Some(format!("TX-CONCURRENT-{:03}", client_id)),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                }),
            };

            // Send and receive
            client.send_request(&request)
                .await
                .expect(&format!("Client {} send should succeed", client_id));
            let response: Casp002Document = client.receive_response()
                .await
                .expect(&format!("Client {} receive should succeed", client_id));

            // Verify response
            assert!(response.document.is_some());
            let doc = response.document.unwrap();
            assert!(doc.sale_to_poi_svc_rsp.is_some());

            // Disconnect
            client.disconnect().await.expect("Disconnect should succeed");

            client_id
        });
        tasks.push(task);
    }

    // Wait for all clients to complete
    let mut completed_clients = Vec::new();
    for task in tasks {
        let client_id = task.await.expect("Task should complete successfully");
        completed_clients.push(client_id);
    }

    // Verify all clients completed
    assert_eq!(completed_clients.len(), num_clients);
}

// ============================================================================
// E2E Test: Raw FramedTransport Client-Server Communication
// ============================================================================

#[tokio::test]
async fn test_e2e_raw_framed_transport_communication() {
    // Create server with handler
    let handler = Arc::new(TestPaymentHandler::new());
    let server = NexoServer::bind("127.0.0.1:0").await.expect("Server bind should succeed");
    let server_addr = server.local_addr().expect("Should get server address");

    // Start server in background
    tokio::spawn(async move {
        server.with_handler(handler).run().await.expect("Server run should succeed");
    });

    // Wait for server to be ready
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Connect with raw TcpStream and wrap in FramedTransport
    let stream = tokio::net::TcpStream::connect(&server_addr.to_string())
        .await
        .expect("Should connect to server");
    let transport = TokioTransport::new(stream);
    let mut framed = FramedTransport::new(transport);

    // Create a payment request
    let request = Casp001Document {
        document: Some(Casp001DocumentDocument {
            sale_to_poi_svc_req: Some(SaleToPoiServiceRequestV06 {
                hdr: Some(Header4 {
                    msg_fctn: Some("DREQ".to_string()),
                    proto_vrsn: Some("6.0".to_string()),
                    tx_id: Some("TX-RAW-FRAMED".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
        }),
    };

    // Send with proper framing
    framed.send_message(&request)
        .await
        .expect("Send should succeed");

    // Receive response with proper framing
    let response: Casp002Document = timeout(Duration::from_secs(5), framed.recv_message())
        .await
        .expect("Should receive within timeout")
        .expect("Receive should succeed");

    // Verify response
    assert!(response.document.is_some());
    let doc = response.document.unwrap();
    assert!(doc.sale_to_poi_svc_rsp.is_some());
    let svc_rsp = doc.sale_to_poi_svc_rsp.unwrap();
    assert!(svc_rsp.hdr.is_some());
    let hdr = svc_rsp.hdr.unwrap();
    assert_eq!(hdr.msg_fctn, Some("DREQ".to_string()));
    assert_eq!(hdr.tx_id, Some("TX-RAW-FRAMED".to_string()));
}

// ============================================================================
// E2E Test: Length Prefix Framing Verification
// ============================================================================

#[tokio::test]
async fn test_e2e_length_prefix_framing_on_wire() {
    // This test verifies that the 4-byte big-endian length prefix
    // is correctly applied on the wire.

    // Create server with handler
    let handler = Arc::new(TestPaymentHandler::new());
    let server = NexoServer::bind("127.0.0.1:0").await.expect("Server bind should succeed");
    let server_addr = server.local_addr().expect("Should get server address");

    // Start server in background
    tokio::spawn(async move {
        server.with_handler(handler).run().await.expect("Server run should succeed");
    });

    // Wait for server to be ready
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Connect with raw TcpStream (no framing layer)
    let mut stream = tokio::net::TcpStream::connect(&server_addr.to_string())
        .await
        .expect("Should connect to server");

    // Create a payment request and encode it
    let request = Casp001Document {
        document: Some(Casp001DocumentDocument {
            sale_to_poi_svc_req: Some(SaleToPoiServiceRequestV06 {
                hdr: Some(Header4 {
                    msg_fctn: Some("DREQ".to_string()),
                    proto_vrsn: Some("6.0".to_string()),
                    tx_id: Some("TX-FRAMING-CHECK".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
        }),
    };

    // Manually add length prefix and send
    use prost::Message;
    let encoded = request.encode_to_vec();
    let mut framed_message = Vec::new();
    framed_message.extend_from_slice(&(encoded.len() as u32).to_be_bytes());
    framed_message.extend_from_slice(&encoded);

    // Send the framed message
    use tokio::io::AsyncWriteExt;
    stream.write_all(&framed_message).await.expect("Write should succeed");

    // Read the response with length prefix
    use tokio::io::AsyncReadExt;
    let mut length_buf = [0u8; 4];
    stream.read_exact(&mut length_buf).await.expect("Should read length prefix");
    let response_length = u32::from_be_bytes(length_buf) as usize;

    // Read the response body
    let mut response_buf = vec![0u8; response_length];
    stream.read_exact(&mut response_buf).await.expect("Should read response body");

    // Decode response
    let response: Casp002Document = Casp002Document::decode(&*response_buf)
        .expect("Should decode response");

    // Verify response
    assert!(response.document.is_some());
    let doc = response.document.unwrap();
    assert!(doc.sale_to_poi_svc_rsp.is_some());
}
