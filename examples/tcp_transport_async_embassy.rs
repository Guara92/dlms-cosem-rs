//! Asynchronous TCP transport example with Embassy runtime for DLMS/COSEM.
//!
//! This example demonstrates how to use the asynchronous TCP transport with
//! Embassy runtime to connect to a DLMS meter and perform basic operations.
//!
//! Embassy is an embedded-first async runtime designed for no_std environments,
//! but can also be used in std environments with appropriate executors.
//!
//! Required features: `async-client`, `transport-tcp-async`, `embassy`
//!
//! # Usage
//!
//! ```bash
//! cargo run --example tcp_transport_async_embassy --features async-client,transport-tcp-async,embassy,encode,parse
//! ```
//!
//! # Note
//!
//! This is a demonstrative example showing API usage patterns.
//! For actual meter communication, replace the mock server with a real DLMS device.
//! In embedded environments, Embassy would use embassy-net instead of std::net.

#[cfg(all(feature = "async-client", feature = "transport-tcp-async", feature = "embassy"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use dlms_cosem::async_client::AsyncClientBuilder;
    use dlms_cosem::client::ClientSettings;
    use dlms_cosem::transport::tcp::EmbassyTcpTransport;
    use std::time::Duration;

    println!("=== DLMS Async TCP Transport Example (Embassy) ===\n");

    // Connection parameters
    let server_address = "192.168.1.100:4059";
    let client_address = 16; // Client SAP
    let server_sap = 1; // Server SAP (logical device address)

    println!("Runtime: Embassy (embedded-first async)");
    println!("Note: In embedded environments, use embassy-net instead of std::net\n");

    // Embassy executor setup
    // In std environments, we use a simple executor
    // In embedded (no_std), you would use embassy-executor
    println!("1. Creating async TCP transport with Embassy...");
    println!("   Server: {}", server_address);
    println!("   Runtime: Embassy");

    // For this example, we'll use futures-executor for simplicity
    // In a real embedded environment, you'd use embassy-executor
    futures_executor::block_on(async {
        // Create async TCP transport with Embassy
        // Note: This will fail without a real DLMS server
        match EmbassyTcpTransport::connect(server_address).await {
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

                // Option 2: Stack-allocated buffer (compile-time size, ideal for embedded/no_std)
                // let mut client = AsyncClientBuilder::new(transport, settings)
                //     .build_with_heapless::<2048>();

                println!("   ✓ Async client created (Embassy runtime)");
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

        println!("\n=== Example complete ===\n");

        // Example usage patterns
        println!("API Usage Patterns (Embassy - std environment):");
        println!("------------------------------------------------");
        println!("use futures_executor::block_on;");
        println!();
        println!("fn main() -> Result<(), Box<dyn std::error::Error>> {{");
        println!("    block_on(async {{");
        println!("        // Create async TCP transport with Embassy");
        println!(
            "        let mut transport = EmbassyTcpTransport::connect(\"192.168.1.100:4059\").await?;"
        );
        println!();
        println!("        // Configure timeouts");
        println!("        transport.set_read_timeout(Some(Duration::from_secs(30)));");
        println!("        transport.set_write_timeout(Some(Duration::from_secs(30)));");
        println!();
        println!("        // Create async client");
        println!("        let settings = ClientSettings {{");
        println!("            client_address: 16,");
        println!("            server_address: 1,");
        println!("            ..Default::default()");
        println!("        }};");
        println!("        // Build client with heap buffer (2048 bytes)");
        println!("        let mut client = AsyncClientBuilder::new(transport, settings)");
        println!("            .build_with_heap(2048);");
        println!();
        println!("        // Or with heapless buffer (RECOMMENDED for embedded):");
        println!("        // let mut client = AsyncClientBuilder::new(transport, settings)");
        println!("        //     .build_with_heapless::<2048>();");
        println!();
        println!("        // Connect and read data (all async with .await)");
        println!("        client.connect().await?;");
        println!("        let data = client.read(class_id, obis_code, attribute_id, None).await?;");
        println!("        client.disconnect().await?;");
        println!("        Ok::<(), Box<dyn std::error::Error>>(())");
        println!("    }})?;");
        println!("    Ok(())");
        println!("}}");
        println!();
        println!("API Usage Patterns (Embassy - no_std embedded):");
        println!("------------------------------------------------");
        println!("#![no_std]");
        println!("#![no_main]");
        println!();
        println!("use embassy_executor::Spawner;");
        println!("use embassy_net::{{Stack, TcpSocket}};");
        println!();
        println!("#[embassy_executor::main]");
        println!("async fn main(spawner: Spawner) {{");
        println!("    // Initialize embassy-net stack (device-specific)");
        println!("    let stack = init_network_stack(spawner).await;");
        println!();
        println!("    // Create embassy-net TCP socket");
        println!("    let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);");
        println!();
        println!("    // Connect to DLMS meter");
        println!("    socket.connect((Ipv4Address::new(192, 168, 1, 100), 4059)).await?;");
        println!();
        println!("    // Wrap in embassy transport (would need embassy-net integration)");
        println!("    // let transport = EmbassyNetTcpTransport::new(socket);");
        println!();
        println!("    // Create DLMS client with heapless buffer (no heap in embedded)");
        println!("    let settings = ClientSettings {{");
        println!("        client_address: 16,");
        println!("        server_address: 1,");
        println!("        ..Default::default()");
        println!("    }};");
        println!("    let mut client = AsyncClientBuilder::new(transport, settings)");
        println!("        .build_with_heapless::<2048>(); // Stack-allocated buffer");
        println!();
        println!("    // Use client (same API as std)");
        println!("    client.connect().await?;");
        println!("    let data = client.read(class_id, obis_code, attribute_id, None).await?;");
        println!("    client.disconnect().await?;");
        println!("}}");
        println!();
        println!("Benefits of Async with Embassy:");
        println!("--------------------------------");
        println!("✓ Designed for embedded systems (no_std)");
        println!("✓ Minimal memory footprint");
        println!("✓ Zero-cost async (no heap allocations for futures)");
        println!("✓ Perfect for microcontrollers and IoT devices");
        println!("✓ Supports both std and no_std environments");
        println!("✓ Cooperative multitasking (single-threaded)");
        println!("✓ Hardware-agnostic (works on ARM, RISC-V, x86)");
        println!();
        println!("Buffer Allocation Options:");
        println!("---------------------------");
        println!("✓ Heap buffer: .build_with_heap(size) - std environments only");
        println!("✓ Stack buffer: .build_with_heapless::<SIZE>() - REQUIRED for no_std");
        println!("  Recommended sizes: 256 (minimal), 2048 (standard), 8192 (load profiles)");
        println!();
        println!("When to Use Embassy:");
        println!("--------------------");
        println!("✓ Embedded systems (MCUs with limited resources)");
        println!("✓ IoT devices with DLMS meters");
        println!("✓ no_std environments");
        println!("✓ Battery-powered devices (low power consumption)");
        println!("✓ Real-time systems (deterministic scheduling)");
        println!("✗ High-concurrency servers (use Tokio instead)");
        println!("✗ CPU-bound workloads (use Tokio with work stealing)");

        Ok::<(), Box<dyn std::error::Error>>(())
    })?;

    Ok(())
}

#[cfg(not(all(feature = "async-client", feature = "transport-tcp-async", feature = "embassy")))]
fn main() {
    eprintln!(
        "This example requires the 'async-client', 'transport-tcp-async', and 'embassy' features."
    );
    eprintln!(
        "Run with: cargo run --example tcp_transport_async_embassy --features async-client,transport-tcp-async,embassy,encode,parse"
    );
    eprintln!();
    eprintln!("Platform Support:");
    eprintln!("  - std environments: Uses std::net::TcpStream in non-blocking mode");
    eprintln!("  - no_std environments: Would use embassy-net (requires additional integration)");
    std::process::exit(1);
}
