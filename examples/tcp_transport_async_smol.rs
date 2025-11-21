//! Asynchronous TCP transport example with Smol runtime for DLMS/COSEM.
//!
//! This example demonstrates how to use the asynchronous TCP transport with
//! Smol runtime to connect to a DLMS meter and perform basic operations.
//!
//! Smol is a lightweight async runtime, perfect for embedded concentrators
//! and resource-constrained systems.
//!
//! Required features: `async-client`, `transport-tcp-async`, `smol`
//!
//! # Usage
//!
//! ```bash
//! cargo run --example tcp_transport_async_smol --features async-client,transport-tcp-async,smol,encode,parse
//! ```
//!
//! # Note
//!
//! This is a demonstrative example showing API usage patterns.
//! For actual meter communication, replace the mock server with a real DLMS device.

#[cfg(all(feature = "async-client", feature = "transport-tcp-async", feature = "smol"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use dlms_cosem::async_client::AsyncClientBuilder;
    use dlms_cosem::client::ClientSettings;
    use dlms_cosem::transport::tcp::SmolTcpTransport;
    use std::time::Duration;

    println!("=== DLMS Async TCP Transport Example (Smol) ===\n");

    // Connection parameters
    let server_address = "192.168.1.100:4059";
    let client_address = 16; // Client SAP
    let server_sap = 1; // Server SAP (logical device address)

    println!("1. Creating async TCP transport with Smol...");
    println!("   Server: {}", server_address);
    println!("   Runtime: Smol (lightweight)");

    // Run async code with Smol runtime
    smol::block_on(async {
        // Create async TCP transport with Smol
        // Note: This will fail without a real DLMS server
        match SmolTcpTransport::connect(server_address).await {
            Ok(mut transport) => {
                println!("   ✓ TCP connection established");

                // Configure timeouts
                transport.set_read_timeout(Some(Duration::from_secs(30)));
                transport.set_write_timeout(Some(Duration::from_secs(30)));

                println!("   ✓ Timeouts configured (30 seconds)");

                // Display connection info
                if let Ok(local_addr) = transport.local_addr() {
                    println!("   Local address: {}", local_addr);
                }
                if let Ok(peer_addr) = transport.peer_addr() {
                    println!("   Peer address: {}", peer_addr);
                }

                println!("\n2. Creating async DLMS client...");

                // Configure client settings
                let settings = ClientSettings {
                    client_address,
                    server_address: server_sap,
                    ..Default::default()
                };

                // Create async DLMS client with TCP transport using builder
                // Option 1: Heap-allocated buffer (runtime size)
                let mut client = AsyncClientBuilder::new(transport, settings).build_with_heap(2048);

                // Option 2: Stack-allocated buffer (compile-time size, for embedded/no_std)
                // let mut client = AsyncClientBuilder::new(transport, settings)
                //     .build_with_heapless::<2048>();

                println!("   ✓ Async client created (Smol runtime)");
                println!("   Client address: {}", client_address);
                println!("   Server address: {}", server_sap);

                println!("\n3. Establishing association...");
                // Note: This will fail without a real server
                match client.connect().await {
                    Ok(_) => {
                        println!("   ✓ Association established");

                        // Example: Read clock object (0.0.1.0.0.255, attribute 2)
                        println!("\n4. Reading clock...");
                        // match client.read_clock().await {
                        //     Ok(datetime) => {
                        //         println!("   ✓ Clock: {:?}", datetime);
                        //     }
                        //     Err(e) => {
                        //         println!("   ✗ Failed to read clock: {}", e);
                        //     }
                        // }

                        println!("\n5. Disconnecting...");
                        match client.disconnect().await {
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

        Ok::<(), Box<dyn std::error::Error>>(())
    })?;

    println!("\n=== Example complete ===\n");

    // Example usage patterns
    println!("API Usage Patterns (Smol):");
    println!("--------------------------");
    println!("use smol::block_on;");
    println!();
    println!("fn main() -> Result<(), Box<dyn std::error::Error>> {{");
    println!("    smol::block_on(async {{");
    println!("        // Create async TCP transport with Smol");
    println!(
        "        let mut transport = SmolTcpTransport::connect(\"192.168.1.100:4059\").await?;"
    );
    println!();
    println!("        // Configure timeouts");
    println!("        transport.set_read_timeout(Some(Duration::from_secs(30)));");
    println!("        transport.set_write_timeout(Some(Duration::from_secs(30)));");
    println!();
    println!("        // Create async client");
    println!("        let settings = ClientSettings {{");
    println!("            client_address: 16,");
    println!("        server_address: 1,");
    println!("        ..Default::default()");
    println!("    }};");
    println!("    // Build client with heap buffer (2048 bytes)");
    println!("    let mut client = AsyncClientBuilder::new(transport, settings)");
    println!("        .build_with_heap(2048);");
    println!();
    println!("    // Or with heapless buffer for embedded systems:");
    println!("    // let mut client = AsyncClientBuilder::new(transport, settings)");
    println!("    //     .build_with_heapless::<2048>();");
    println!();
    println!("        // Connect and read data (all async with .await)");
    println!("        client.connect().await?;");
    println!("        let data = client.read(class_id, obis_code, attribute_id, None).await?;");
    println!("        client.disconnect().await?;");
    println!("        Ok(())");
    println!("    }})");
    println!("}}");
    println!();
    println!("Benefits of Smol Runtime:");
    println!("-------------------------");
    println!("✓ Lightweight (~100KB binary size impact)");
    println!("✓ Perfect for embedded concentrators (100-1,000 meters)");
    println!("✓ Low memory footprint");
    println!("✓ Simple API (block_on for simple cases)");
    println!("✓ Good performance with moderate concurrency");
    println!("✓ Cross-platform (Linux, Windows, macOS)");
    println!();
    println!("Comparison: Smol vs Tokio:");
    println!("--------------------------");
    println!("Smol:   Small binary, moderate scale, embedded systems");
    println!("Tokio:  Large binary, massive scale, HES/SaaS platforms");

    Ok(())
}

#[cfg(not(all(feature = "async-client", feature = "transport-tcp-async", feature = "smol")))]
fn main() {
    eprintln!(
        "This example requires the 'async-client', 'transport-tcp-async', and 'smol' features."
    );
    eprintln!(
        "Run with: cargo run --example tcp_transport_async_smol --features async-client,transport-tcp-async,smol,encode,parse"
    );
    std::process::exit(1);
}
