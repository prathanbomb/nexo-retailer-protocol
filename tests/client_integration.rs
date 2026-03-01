//! Integration tests for Nexo Retailer Protocol Client
//!
//! This test suite verifies the complete client request/response flow with a mock Nexo server.

mod mock_server;

use std::time::Duration;
use tokio::time::timeout;

use nexo_retailer_protocol::{
    NexoClient, ReconnectConfig, TimeoutConfig, MessageBuilder,
    Header4Builder, PaymentRequestBuilder,
};

// ============================================================================
// Task 1: Mock Server Tests
// ============================================================================

#[tokio::test]
async fn mock_server_starts() {
    // Initialize logger for test output
    let _ = env_logger::try_init();

    let server = mock_server::MockNexoServer::start()
        .await
        .expect("Failed to start mock server");

    // Spawn server to run in background
    let server_clone = server.clone();
    tokio::spawn(async move {
        let _ = server_clone.run().await;
    });

    // Give server time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Verify server address is valid
    let addr = server.addr();
    assert!(addr.port() > 0, "Server should bind to non-zero port");

    // Stop server
    server.stop().await;
}

// ============================================================================
// Task 2: Client Connection and Basic Request/Response
// ============================================================================

#[tokio::test]
async fn test_client_connects_to_server() {
    let _ = env_logger::try_init();

    // Start mock server
    let server = mock_server::MockNexoServer::start()
        .await
        .expect("Failed to start mock server");

    let server_clone = server.clone();
    tokio::spawn(async move {
        let _ = server_clone.run().await;
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Connect client to server
    let mut client = NexoClient::new();
    let addr = server.addr().to_string();
    client.connect(&addr).await
        .expect("Client should connect to server");

    // Verify connection
    assert!(client.is_connected(), "Client should be connected");

    // Cleanup
    let _ = client.disconnect().await;
    server.stop().await;
}

#[tokio::test]
async fn test_client_sends_payment_request() {
    let _ = env_logger::try_init();

    // Start mock server
    let server = mock_server::MockNexoServer::start()
        .await
        .expect("Failed to start mock server");

    let server_clone = server.clone();
    tokio::spawn(async move {
        let _ = server_clone.run().await;
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Connect client
    let mut client = NexoClient::new();
    let addr = server.addr().to_string();
    client.connect(&addr).await
        .expect("Client should connect");

    // Build a header using builder pattern
    let header = Header4Builder::new()
        .message_function("DREQ".to_string())
        .protocol_version("6.0".to_string())
        .transaction_id("TX-12345".to_string())
        .build()
        .expect("Header should build successfully");

    // Build a payment request using builder pattern
    let payment = PaymentRequestBuilder::new()
        .transaction_id("TX-12345".to_string())
        .build()
        .expect("Payment request should build successfully");

    // Create a Transaction23 wrapper for the payment request
    let transaction = nexo_retailer_protocol::Transaction23 {
        tx_oneof: Some(nexo_retailer_protocol::transaction23::TxOneof::PmtReq(payment)),
    };

    // Create a SaleToPoiServiceRequestV06 with the header and payment transaction
    let service_request = nexo_retailer_protocol::SaleToPoiServiceRequestV06 {
        hdr: Some(header),
        tx: vec![transaction],
        scty_trlr: None,
        login_req: None,
    };

    // Wrap in Casp001Document
    let request = nexo_retailer_protocol::Casp001Document {
        document: Some(nexo_retailer_protocol::Casp001DocumentDocument {
            sale_to_poi_svc_req: Some(service_request),
        }),
    };

    // Send request (we use send_and_receive which returns the same type)
    let _ = client.send_and_receive(&request)
        .await
        .expect("Send should succeed");

    // Cleanup
    let _ = client.disconnect().await;
    server.stop().await;
}

#[tokio::test]
async fn test_client_receives_response() {
    let _ = env_logger::try_init();

    // Start mock server
    let server = mock_server::MockNexoServer::start()
        .await
        .expect("Failed to start mock server");

    let server_clone = server.clone();
    tokio::spawn(async move {
        let _ = server_clone.run().await;
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Connect and send request
    let mut client = NexoClient::new();
    let addr = server.addr().to_string();
    client.connect(&addr).await
        .expect("Client should connect");

    let request = nexo_retailer_protocol::Casp001Document::default();

    // Send request
    client.send_request(&request).await
        .expect("Send should succeed");

    // Receive echoed response
    let response: nexo_retailer_protocol::Casp002Document = client.receive_response()
        .await
        .expect("Should receive response");

    // Verify we got a response (mock server echoes back)
    // The response should be a valid Casp002Document
    assert!(response.document.is_some(), "Response should have document");

    // Cleanup
    let _ = client.disconnect().await;
    server.stop().await;
}
