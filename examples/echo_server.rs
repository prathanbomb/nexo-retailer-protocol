//! Echo Server Example for Nexo Retailer Protocol
//!
//! This example demonstrates how to create a TCP echo server using the
//! Tokio transport implementation. The server accepts connections and
//! echoes back any messages it receives.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example echo_server --features std
//! ```
//!
//! Or with custom bind address:
//!
//! ```bash
//! cargo run --example echo_server --features std -- 0.0.0.0:9000
//! ```
//!
//! # Features Demonstrated
//!
//! - TcpListener binding to specified address
//! - Accept loop for handling incoming connections
//! - Creating TokioTransport from accepted connections
//! - FramedTransport for message handling
//! - Echoing received messages back to client
//! - Graceful shutdown handling (Ctrl+C)
//! - Concurrent connection handling with tokio::spawn
//!
//! # Testing the Server
//!
//! 1. Start the echo server:
//!    ```bash
//!    cargo run --example echo_server --features std
//!    ```
//!
//! 2. In another terminal, run the Tokio client:
//!    ```bash
//!    cargo run --example tokio_client --features std -- 127.0.0.1:8080
//!    ```

#[cfg(feature = "std")]
use core::time::Duration;

#[cfg(feature = "std")]
use tokio::net::TcpListener;

#[cfg(feature = "std")]
use nexo_retailer_protocol::{
    FramedTransport, Header4, NexoError, TokioTransport,
};

#[cfg(feature = "std")]
use tokio::main;

/// Handle a single client connection
///
/// This function is spawned in a separate task for each accepted connection,
/// allowing the server to handle multiple clients concurrently.
///
/// # Arguments
///
/// * `tcp_stream` - The accepted TCP connection
/// * `peer_addr` - The client's socket address
#[cfg(feature = "std")]
async fn handle_client(
    tcp_stream: tokio::net::TcpStream,
    peer_addr: std::net::SocketAddr,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("[{}] Client connected", peer_addr);

    // Step 1: Create TokioTransport from the accepted connection
    // Unlike the client which uses TokioTransport::connect(), the server
    // creates transport from an already-connected TcpStream.
    let transport = TokioTransport::new(tcp_stream);

    // Step 2: Wrap in FramedTransport for message handling
    let mut framed = FramedTransport::new(transport);

    // Step 3: Echo loop - receive messages and send them back
    loop {
        match framed.recv_message::<Header4>().await {
            Ok(message) => {
                println!(
                    "[{}] Received message: msg_fctn={:?}, tx_id={:?}",
                    peer_addr,
                    message.msg_fctn,
                    message.tx_id
                );

                // Echo the message back to the client
                if let Err(e) = framed.send_message(&message).await {
                    eprintln!("[{}] Failed to send echo: {}", peer_addr, e);
                    break;
                }

                println!("[{}] Echoed message back to client", peer_addr);
            }
            Err(e) => {
                // Connection closed or error occurred
                match e {
                    NexoError::Connection { details } => {
                        if details.contains("unexpected EOF") || details.contains("closed") {
                            println!("[{}] Client disconnected", peer_addr);
                        } else {
                            eprintln!("[{}] Connection error: {}", peer_addr, details);
                        }
                    }
                    _ => {
                        eprintln!("[{}] Error receiving message: {}", peer_addr, e);
                    }
                }
                break;
            }
        }
    }

    println!("[{}] Client handler finished", peer_addr);
    Ok(())
}

/// Main server entry point
#[cfg(feature = "std")]
#[main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Parse command-line arguments
    let args: Vec<String> = std::env::args().collect();

    let bind_addr = if args.len() > 1 {
        // User provided address
        args[1].clone()
    } else {
        // Default address for local testing
        println!("Usage: {} [bind_address]", args[0]);
        println!("Using default address: 127.0.0.1:8080");
        "127.0.0.1:8080".to_string()
    };

    println!("Nexo Retailer Protocol - Echo Server Example");
    println!("Binding to: {}", bind_addr);

    // Step 1: Create TCP listener
    // The TcpListener listens for incoming TCP connections on the specified address.
    println!("\n[1] Creating TCP listener...");
    let listener = TcpListener::bind(&bind_addr).await.map_err(|e| {
        eprintln!("✗ Failed to bind to {}: {}", bind_addr, e);
        eprintln!("  The address may be in use or invalid");
        Box::new(e) as Box<dyn std::error::Error + Send + Sync>
    })?;
    println!("✓ Server listening on {}", bind_addr);

    // Step 2: Get the actual bound address (useful if binding to port 0)
    let actual_addr = listener.local_addr()?;
    println!("✓ Actual bound address: {}", actual_addr);

    println!("\n[2] Server ready. Press Ctrl+C to stop.");
    println!("\nWaiting for connections...\n");

    // Step 3: Accept loop
    // The server will accept connections indefinitely (until Ctrl+C)
    let mut connection_count = 0u32;

    loop {
        // Accept a new connection
        match listener.accept().await {
            Ok((tcp_stream, peer_addr)) => {
                connection_count += 1;
                println!("--- Connection #{} ---", connection_count);
                println!("Accepted connection from {}", peer_addr);

                // Spawn a new task to handle this client
                // This allows the server to handle multiple clients concurrently
                tokio::spawn(async move {
                    if let Err(e) = handle_client(tcp_stream, peer_addr).await {
                        eprintln!("[{}] Client handler error: {}", peer_addr, e);
                    }
                    println!("[{}] Client handler task completed", peer_addr);
                });
            }
            Err(e) => {
                eprintln!("✗ Error accepting connection: {}", e);
                // Continue accepting other connections
            }
        }
    }

    // Note: The code above never reaches this point due to the infinite loop.
    // For a production server, you would implement graceful shutdown handling
    // using tokio::signal (requires adding "signal" feature to tokio).
    //
    // Example graceful shutdown pattern:
    // ```rust,ignore
    // use tokio::signal;
    //
    // let ctrl_c = signal::ctrl_c();
    // tokio::pin!(ctrl_c);
    //
    // loop {
    //     tokio::select! {
    //         result = listener.accept() => { /* handle connection */ }
    //         _ = &mut ctrl_c => {
    //             println!("Shutdown signal received");
    //             break;
    //         }
    //     }
    // }
    // ```

    Ok(())
}

/// Simple echo server example (alternative version)
///
/// This version shows a simpler pattern without concurrent connection handling.
/// Useful for understanding the basic server pattern before adding complexity.
#[cfg(feature = "std")]
#[allow(dead_code)]
async fn simple_echo_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use nexo_retailer_protocol::{FramedTransport, TokioTransport};

    let bind_addr = "127.0.0.1:8080";
    println!("Simple echo server on {}", bind_addr);

    let listener = TcpListener::bind(bind_addr).await?;
    println!("Listening...");

    // Accept a single connection
    let (tcp_stream, peer_addr) = listener.accept().await?;
    println!("Connected to {}", peer_addr);

    // Create transport from accepted stream
    let transport = TokioTransport::new(tcp_stream);
    let mut framed = FramedTransport::new(transport);

    // Echo loop (single connection)
    loop {
        match framed.recv_message::<Header4>().await {
            Ok(message) => {
                println!("Received: {:?}", message.msg_fctn);
                framed.send_message(&message).await?;
                println!("Echoed back");
            }
            Err(_) => {
                println!("Connection closed");
                break;
            }
        }
    }

    Ok(())
}
