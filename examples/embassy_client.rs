//! Embassy Client Example for Nexo Retailer Protocol
//!
//! This example demonstrates how to use the Embassy transport implementation
//! in embedded (no_std) environments to connect to a Nexo server.
//!
//! # Usage
//!
//! ```bash
//! # Build for bare-metal target (requires appropriate target installed)
//! cargo build --example embassy_client --features embassy --target thumbv7em-none-eabihf
//! ```
//!
//! # Features Demonstrated
//!
//! - Embassy executor setup with entry macro
//! - Static buffer allocation for Embassy TCP socket
//! - EmbassyTransport creation with buffer lifetime management
//! - Timeout configuration for embedded environments
//! - FramedTransport wrapping for message framing
//! - Sending/receiving CASP messages
//! - Embedded-specific error handling
//!
//! # Hardware Requirements
//!
//! This example requires:
//! - Embassy-compatible hardware (e.g., STM32, ESP32, nRF)
//! - Network hardware (Ethernet/WiFi) supported by embassy-net
//! - Or QEMU emulation for testing
//!
//! # Network Stack Setup
//!
//! Embassy requires platform-specific network stack initialization.
//! This example shows the transport usage pattern but comments out
//! the hardware-specific setup code.
//!
//! # Running on Hardware
//!
//! 1. Install the appropriate target:
//!    ```bash
//!    rustup target add thumbv7em-none-eabihf
//!    ```
//!
//! 2. Build for your target:
//!    ```bash
//!    cargo build --example embassy_client --features embassy --target thumbv7em-none-eabihf
//!    ```
//!
//! 3. Flash to your hardware using appropriate tools (probe-rs, cargo-flash, etc.)
//!
//! # Running with QEMU
//!
//! Some Embassy examples can run with QEMU emulation. Check Embassy documentation
//! for your specific hardware platform.

#![cfg(feature = "embassy-net")]

use embassy_time::Duration;

// NOTE: In a real application, you would import:
// use embassy_net::TcpSocket;
// use nexo_retailer_protocol::{
//     EmbassyTransport, EmbassyTimeoutConfig, FramedTransport, Header4,
// };
//
// However, embassy-net's TcpSocket requires a network stack that we can't
// initialize in this example without hardware. The code below demonstrates
// the usage pattern with comments explaining each step.

/// Embassy main entry point
///
/// The `#[embassy_executor::main]` macro sets up the Embassy executor
/// and provides the async context for the application.
///
/// # Spawner
///
/// The spawner parameter allows spawning additional tasks. For this
/// simple example, we run everything sequentially in the main task.
///
/// NOTE: The exact macro syntax depends on your Embassy version and
/// target platform. Common variants:
/// - #[embassy_executor::main] (most platforms)
/// - #[embassy_executor::main(entry = "main")] (some platforms)
/// - Check Embassy documentation for your specific hardware
///
/// NOTE: This is commented out because it requires target-specific
/// configuration. Uncomment in your actual application.
/*
#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
*/

/// Dummy main for compilation
/// In your actual application, use the #[embassy_executor::main] macro above
fn main() {
    // ===== EMBASSY EXECUTOR SETUP =====
    // The #[embassy_executor::main] macro has already set up the executor.
    // Now we need to initialize the network stack (hardware-specific).

    // ===== STATIC BUFFER ALLOCATION =====
    // Embassy TCP sockets require statically allocated buffers.
    // The buffer size determines the maximum transmission unit (MTU).
    // Common values: 1536 bytes (standard Ethernet), 4096 bytes (more headroom).
    //
    // NOTE: These buffers must live for the entire lifetime of the socket.
    // That's why EmbassyTransport has a lifetime parameter tied to the buffers.

    // Example buffer declarations (commented out to avoid static mut issues):
    // static mut RX_BUFFER: [u8; 4096] = [0u8; 4096];
    // static mut TX_BUFFER: [u8; 4096] = [0u8; 4096];

    // ===== NETWORK STACK INITIALIZATION (HARDWARE-SPECIFIC) =====
    // This section is highly hardware-dependent and would typically include:
    //
    // 1. Initialize network hardware (Ethernet MAC, WiFi controller)
    //    Example for STM32 Ethernet:
    //    ```ignore
    //    let mut embassy_device = unsafe { stm32_eth::Ethernet::default() };
    //    ```
    //
    // 2. Create network stack with DHCP or static IP configuration
    //    Example:
    //    ```ignore
    //    let config = embassy_net::Config::dhcpv4(Default::default());
    //    // or
    //    let config = embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
    //        address: Ipv4Cidr::new(Ipv4Address::new(192, 168, 1, 100), 24),
    //        dns_servers: Vec::new(),
    //        gateway: Some(Ipv4Address::new(192, 168, 1, 1)),
    //    });
    //
    //    let stack = embassy_net::Stack::new(
    //        embassy_device,
    //        embassy_rx_buffer,
    //        embassy_tx_buffer,
    //        &mut embassy_seed,
    //        config,
    //    );
    //    ```
    //
    // 3. Wait for network stack to be up (DHCP lease, link up, etc.)
    //    Example:
    //    ```ignore
    //    stack.wait_config_up().await;
    //    ```

    // For this example, we'll demonstrate the transport usage pattern
    // without actual hardware initialization. In a real application,
    // you would have a `stack` variable here from the initialization above.

    // ===== CREATE TCP SOCKET =====
    // TcpSocket is created from the network stack and binds to our buffers.
    // The buffers are borrowed for the socket's lifetime.
    //
    // Example (with actual stack):
    // ```ignore
    // let mut rx_buffer = unsafe { &mut *core::ptr::addr_of_mut!(RX_BUFFER) };
    // let mut tx_buffer = unsafe { &mut *core::ptr::addr_of_mut!(TX_BUFFER) };
    // let socket = TcpSocket::new(stack, rx_buffer, tx_buffer);
    // ```

    // ===== CREATE EMBASSY TRANSPORT =====
    // EmbassyTransport wraps the TcpSocket and provides the Transport trait.
    // It requires:
    // - The socket
    // - Mutable references to RX and TX buffers (for lifetime tracking)
    //
    // Default timeouts:
    // - Connect: 10 seconds
    // - Read: 30 seconds
    // - Write: 10 seconds
    //
    // Example:
    // ```ignore
    // let mut transport = EmbassyTransport::new(socket, rx_buffer, tx_buffer)
    //     .with_timeouts(Duration::from_secs(30), Duration::from_secs(10));
    // ```

    // ===== CONNECT TO REMOTE SERVER =====
    // The connect() method takes an address string "IP:PORT" format.
    // Embassy parses the address into (IpAddress, u16) tuple internally.
    //
    // Example:
    // ```ignore
    // if let Err(e) = transport.connect("192.168.1.100:8080").await {
    //     // Handle connection error
    //     // In embedded systems, you might want to retry or indicate failure via LED
    // }
    // ```

    // ===== WRAP IN FRAMED TRANSPORT =====
    // FramedTransport adds length-prefixed framing (4-byte big-endian).
    // This is the same for both Tokio and Embassy implementations.
    //
    // Example:
    // ```ignore
    // let mut framed = FramedTransport::new(transport);
    // ```

    // ===== CREATE CASP MESSAGE =====
    // Creating messages is identical to std environment.
    // All CASP message types are available in no_std.
    //
    // Example:
    // ```ignore
    // let header = Header4 {
    //     msg_fctn: Some("DREQ".to_string()),
    //     proto_vrsn: Some("6.0".to_string()),
    //     tx_id: Some("EMBEDDED-TX-001".to_string()),
    //     ..Default::default()
    // };
    // ```

    // ===== SEND MESSAGE =====
    // Message sending works identically to std environment.
    // FramedTransport handles encoding and length-prefix framing.
    //
    // Example:
    // ```ignore
    // framed.send_message(&header).await?;
    // ```

    // ===== RECEIVE MESSAGE =====
    // Message receiving works identically to std environment.
    // FramedTransport handles length-prefix parsing and decoding.
    //
    // Example:
    // ```ignore
    // let response: Header4 = framed.recv_message().await?;
    // ```

    // ===== EMBEDDED-SPECIFIC CONSIDERATIONS =====
    //
    // 1. **Buffer Management**
    //    - All buffers must be statically allocated (no heap allocation)
    //    - Buffer lifetimes are tracked at compile time
    //    - Buffer size determines maximum message size
    //
    // 2. **Timeout Configuration**
    //    - Use embassy_time::Duration (not core::time::Duration)
    //    - Timeouts are enforced using embassy_futures::select
    //    - Adjust timeouts based on your network conditions
    //
    // 3. **Error Handling**
    //    - NexoError works in no_std (core::error::Error)
    //    - Consider retry logic for transient failures
    //    - May want to signal errors via hardware (LED, buzzer)
    //
    // 4. **Resource Cleanup**
    //    - Embasdy uses RAII for resource management
    //    - Transport/sockets are automatically closed when dropped
    //    - No explicit cleanup needed in most cases
    //
    // 5. **Concurrency**
    //    - Use embassy_executor::Spawner for background tasks
    //    - Example: spawn task for periodic status updates
    //    - Example: spawn task for watching connection state

    // ===== COMPILATION NOTE =====
    // This example demonstrates the Embassy transport usage pattern but
    // cannot compile without actual hardware or QEMU setup due to
    // embassy-net dependencies.
    //
    // For a complete working example, see the Embassy repository examples
    // for your specific hardware platform (STM32, ESP32, nRF, etc.).
    //
    // The commented code above shows the exact usage pattern you would use
    // in a real embedded application.

    #[cfg(feature = "defmt")]
    defmt::warn!("This example demonstrates usage patterns but requires hardware to run");
    #[cfg(not(feature = "defmt"))]
    {
        // In a real scenario, you'd initialize the network stack here
    }

    // ===== SUMMARY =====
    //
    // This example demonstrates the Embassy transport usage pattern:
    // 1. Allocate static buffers for TCP socket
    // 2. Initialize network stack (hardware-specific)
    // 3. Create TcpSocket from stack
    // 4. Create EmbassyTransport with buffers and socket
    // 5. Connect to remote address
    // 6. Wrap in FramedTransport for message framing
    // 7. Send/receive CASP messages
    //
    // The key difference from Tokio is the need for static buffer allocation
    // and hardware-specific network stack initialization.
    //
    // For complete working examples, see the Embassy repository examples
    // for your specific hardware platform.
}

/// Example of timeout configuration with EmbassyTimeoutConfig
///
/// EmbassyTimeoutConfig provides the same builder pattern as Tokio's TimeoutConfig
/// but uses embassy_time::Duration instead of core::time::Duration.
///
/// NOTE: This function is commented out because it references types that require
/// full Embassy setup. Uncomment and use in your actual application.
/*
#[cfg(feature = "embassy-net")]
fn example_timeout_config() {
    use embassy_time::Duration;
    use nexo_retailer_protocol::EmbassyTimeoutConfig;

    let config = EmbassyTimeoutConfig::new()
        .with_connect(Duration::from_secs(5))  // Shorter connect timeout
        .with_read(Duration::from_secs(60))     // Longer read timeout for slow networks
        .with_write(Duration::from_secs(20));   // Moderate write timeout

    // You can then use this config with the timeout wrapper methods:
    // async fn example() -> Result<(), nexo_retailer_protocol::NexoError> {
    //     config.with_connect_timeout(async {
    //         // Connection logic
    //         Ok(())
    //     }).await?;
    //     Ok(())
    // }

    let _ = config; // Silence unused warning
}
*/

/// Example of buffer size considerations
///
/// The buffer size you choose affects:
/// - Maximum message size you can receive
/// - Memory usage (critical in embedded systems)
/// - Performance (too small = more read calls)
///
/// NOTE: This is documentation for buffer sizing strategy.
fn example_buffer_size_considerations() {
    // Small buffer (1.5 KB) - suitable for simple messages
    // Uses less RAM but may require multiple read calls for large messages
    const SMALL_BUFFER_SIZE: usize = 1536;

    // Medium buffer (4 KB) - good balance for most CASP messages
    // Reasonable memory usage, handles most payment messages
    const MEDIUM_BUFFER_SIZE: usize = 4096;

    // Large buffer (8 KB) - for complex messages with lots of data
    // Uses more RAM, fewer read calls
    const LARGE_BUFFER_SIZE: usize = 8192;

    // Choose based on:
    // - Expected message size in your application
    // - Available RAM on your microcontroller
    // - Performance requirements

    let _ = (SMALL_BUFFER_SIZE, MEDIUM_BUFFER_SIZE, LARGE_BUFFER_SIZE);
}
