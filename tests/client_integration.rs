//! Integration tests for Nexo Retailer Protocol Client
//!
//! This test suite verifies the complete client request/response flow with a mock Nexo server.

mod mock_server;

use std::time::Duration;
use tokio::time::timeout;

use nexo_retailer_protocol::{
    NexoClient, ReconnectConfig, TimeoutConfig, MessageBuilder, NexoError,
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

    // Give server time to process and respond
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

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

// ============================================================================
// Task 3: Reconnection Logic and Timeout Handling
// ============================================================================

#[tokio::test]
async fn test_client_reconnects_on_connection_failure() {
    let _ = env_logger::try_init();

    // Start mock server
    let server = mock_server::MockNexoServer::start()
        .await
        .expect("Failed to start mock server");

    let server_addr = server.addr().to_string();

    let server_clone = server.clone();
    tokio::spawn(async move {
        let _ = server_clone.run().await;
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Connect client
    let mut client = NexoClient::new();
    client.connect(&server_addr).await
        .expect("Client should connect");

    assert!(client.is_connected(), "Client should be connected");

    // Stop server (connection lost)
    server.stop().await;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Disconnect client explicitly
    let _ = client.disconnect().await;

    // Restart server on same port
    let server2 = mock_server::MockNexoServer::start()
        .await
        .expect("Failed to restart server");

    let server2_addr = server2.addr().to_string();

    let server2_clone = server2.clone();
    tokio::spawn(async move {
        let _ = server2_clone.run().await;
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Client reconnects successfully
    client.connect(&server2_addr).await
        .expect("Client should reconnect");

    assert!(client.is_connected(), "Client should be reconnected");

    // Cleanup
    let _ = client.disconnect().await;
    server2.stop().await;
}

#[tokio::test]
async fn test_client_times_out_on_slow_response() {
    let _ = env_logger::try_init();

    // Start mock server
    let server = mock_server::MockNexoServer::start()
        .await
        .expect("Failed to start mock server");

    // Set delay to be longer than timeout (200ms delay, 100ms timeout)
    server.set_delay_response(200).await;

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

    // Set short timeout
    let timeout_config = TimeoutConfig::new()
        .with_request_timeout(std::time::Duration::from_millis(100));
    client = client.with_timeout_config(timeout_config);

    let request = nexo_retailer_protocol::Casp001Document::default();

    // Send request with timeout - should timeout
    let result = client.send_with_timeout(&request, std::time::Duration::from_millis(100)).await;

    assert!(result.is_err(), "Should timeout on slow response");
    match result {
        Err(nexo_retailer_protocol::NexoError::Timeout) => {
            // Expected - timeout occurred
        }
        Err(e) => {
            panic!("Expected timeout error, got: {:?}", e);
        }
        Ok(_) => {
            panic!("Expected timeout error, but got success");
        }
    }

    // Cleanup
    let _ = client.disconnect().await;
    server.stop().await;
}

// ============================================================================
// Task 4: Builder Pattern and Message ID Uniqueness
// ============================================================================

#[tokio::test]
async fn test_client_sends_built_message() {
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

    // Send the built message
    client.send_request(&request).await
        .expect("Send should succeed");

    // Cleanup
    let _ = client.disconnect().await;
    server.stop().await;
}

#[tokio::test]
async fn test_builder_rejects_invalid_message() {
    let _ = env_logger::try_init();

    // Try to build a header with missing required fields
    let header_result = Header4Builder::new()
        // Missing message_function, protocol_version, transaction_id
        .build();

    assert!(header_result.is_err(), "Builder should reject invalid message");
    match header_result {
        Err(NexoError::Validation { field, .. }) => {
            assert!(field.contains("msg_fctn") || field.contains("proto_vrsn") || field.contains("tx_id"),
                   "Should validate required fields");
        }
        _ => {
            panic!("Expected validation error, got: {:?}", header_result);
        }
    }
}
