//! Synchronous TCP transport example for DLMS/COSEM.
//!
//! This example demonstrates how to use the synchronous TCP transport
//! to connect to a DLMS meter and perform basic operations.
//!
//! Required features: `client`, `transport-tcp`
//!
//! # Usage
//!
//! ```bash
//! cargo run --example tcp_transport_sync --features client,transport-tcp
//! ```
//!
//! # Note
//!
//! This is a demonstrative example showing API usage patterns.
//! For actual meter communication, replace the mock server with a real DLMS device.

#[cfg(all(feature = "client", feature = "transport-tcp"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use dlms_cosem::client::sync::{ClientBuilder, ClientSettings};
    use dlms_cosem::transport::tcp::TcpTransport;
    use std::time::Duration;

    println!("=== DLMS Synchronous TCP Transport Example ===\n");

    // Connection parameters
    let server_address = "192.168.1.100:4059";
    let client_address = 16; // Client SAP
    let server_sap = 1; // Server SAP (logical device address)

    println!("1. Creating TCP transport...");
    println!("   Server: {}", server_address);

    // Create TCP transport
    // Note: This will fail without a real DLMS server
    match TcpTransport::connect(server_address) {
        Ok(mut transport) => {
            println!("   ✓ TCP connection established");

            // Configure timeouts
            transport
                .set_read_timeout(Some(Duration::from_secs(30)))
                .expect("Failed to set read timeout");
            transport
                .set_write_timeout(Some(Duration::from_secs(30)))
                .expect("Failed to set write timeout");

            println!("   ✓ Timeouts configured (30 seconds)");

            // Display connection info
            if let Ok(local_addr) = transport.local_addr() {
                println!("   Local address: {}", local_addr);
            }
            if let Ok(peer_addr) = transport.peer_addr() {
                println!("   Peer address: {}", peer_addr);
            }

            println!("\n2. Creating DLMS client...");

            // Configure client settings
            let settings =
                ClientSettings { client_address, server_address: server_sap, ..Default::default() };

            // Create DLMS client with TCP transport using builder
            // Option 1: Heap-allocated buffer (runtime size)
            let mut client = ClientBuilder::new(transport, settings).build_with_heap(2048);

            // Option 2: Stack-allocated buffer (compile-time size, for embedded/no_std)
            // let mut client = ClientBuilder::new(transport, settings)
            //     .build_with_heapless::<2048>();

            println!("   ✓ Client created (heap buffer: 2048 bytes)");
            println!("   Client address: {}", client_address);
            println!("   Server address: {}", server_sap);

            println!("\n3. Establishing association...");
            // Note: This will fail without a real server
            match client.connect() {
                Ok(_) => {
                    println!("   ✓ Association established");

                    // Example: Read clock object (0.0.1.0.0.255, attribute 2)
                    println!("\n4. Reading clock...");
                    // match client.read_clock() {
                    //     Ok(datetime) => {
                    //         println!("   ✓ Clock: {:?}", datetime);
                    //     }
                    //     Err(e) => {
                    //         println!("   ✗ Failed to read clock: {}", e);
                    //     }
                    // }

                    println!("\n5. Disconnecting...");
                    match client.disconnect() {
                        Ok(_) => println!("   ✓ Disconnected successfully"),
                        Err(e) => println!("   ✗ Disconnect failed: {}", e),
                    }
                }
                Err(e) => {
                    println!("   ✗ Association failed: {}", e);
                    println!("   (This is expected without a real DLMS server)");
                }
            }
        }
        Err(e) => {
            println!("   ✗ TCP connection failed: {}", e);
            println!("   (This is expected without a real DLMS server)");
            println!("\n   To use this example:");
            println!(
                "   1. Start a DLMS server on {}:4059",
                server_address.split(':').next().unwrap()
            );
            println!("   2. Or modify the server_address to point to your meter");
        }
    }

    println!("\n=== Example complete ===\n");

    // Example usage patterns
    println!("API Usage Patterns:");
    println!("-------------------");
    println!("// Create TCP transport");
    println!("let transport = TcpTransport::connect(\"192.168.1.100:4059\")?;");
    println!();
    println!("// Configure timeouts");
    println!("transport.set_read_timeout(Some(Duration::from_secs(30)))?;");
    println!("transport.set_write_timeout(Some(Duration::from_secs(30)))?;");
    println!();
    println!("// Create client");
    println!("let settings = ClientSettings {{");
    println!("    client_address: 16,");
    println!("    server_address: 1,");
    println!("    ..Default::default()");
    println!("}};");
    println!("// Build client with heap buffer (2048 bytes)");
    println!("let mut client = ClientBuilder::new(transport, settings)");
    println!("    .build_with_heap(2048);");
    println!();
    println!("// Or with heapless buffer for embedded systems:");
    println!("// let mut client = ClientBuilder::new(transport, settings)");
    println!("//     .build_with_heapless::<2048>();");
    println!();
    println!("// Connect and read data");
    println!("client.connect()?;");
    println!("let data = client.read(class_id, obis_code, attribute_id)?;");
    println!("client.disconnect()?;");
    println!();
    println!("Buffer Allocation Options:");
    println!("---------------------------");
    println!("✓ Heap buffer: .build_with_heap(size) - flexible, runtime-determined");
    println!("✓ Stack buffer: .build_with_heapless::<SIZE>() - embedded, compile-time");
    println!("  Recommended sizes: 256 (minimal), 2048 (standard), 8192 (load profiles)");

    Ok(())
}

#[cfg(not(all(feature = "client", feature = "transport-tcp")))]
fn main() {
    eprintln!("This example requires the 'client' and 'transport-tcp' features.");
    eprintln!("Run with: cargo run --example tcp_transport_sync --features client,transport-tcp");
    std::process::exit(1);
}
