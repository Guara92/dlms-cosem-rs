//! Asynchronous TCP transport example with Glommio runtime for DLMS/COSEM.
//!
//! This example demonstrates how to use the asynchronous TCP transport with
//! Glommio runtime to connect to a DLMS meter and perform basic operations.
//!
//! Glommio is a thread-per-core runtime for Linux that uses io_uring for
//! high-performance, low-latency I/O operations.
//!
//! Required features: `async-client`, `transport-tcp-async`, `glommio`
//!
//! # Platform Requirements
//!
//! - Linux kernel 5.8+ with io_uring support
//! - x86_64 or aarch64 architecture
//!
//! # Usage
//!
//! ```bash
//! cargo run --example tcp_transport_async_glommio --features async-client,transport-tcp-async,glommio,encode,parse
//! ```
//!
//! # Note
//!
//! This is a demonstrative example showing API usage patterns.
//! For actual meter communication, replace the mock server with a real DLMS device.

#[cfg(all(feature = "async-client", feature = "transport-tcp-async", feature = "glommio"))]
fn main() -> Result<(), std::io::Error> {
    use dlms_cosem::async_client::AsyncClientBuilder;
    use dlms_cosem::client::ClientSettings;
    use dlms_cosem::transport::tcp::GlommioTcpTransport;
    use glommio::LocalExecutorBuilder;
    use std::time::Duration;

    println!("=== DLMS Async TCP Transport Example (Glommio) ===\n");

    // Connection parameters
    let server_address = "192.168.1.100:4059";
    let client_address = 16; // Client SAP
    let server_sap = 1; // Server SAP (logical device address)

    println!("Platform Requirements:");
    println!("  Linux kernel 5.8+ with io_uring support");
    println!("  Runtime: Glommio (thread-per-core, io_uring-based)\n");

    // Create a Glommio local executor
    // Glommio uses thread-per-core architecture
    let local_ex = LocalExecutorBuilder::default()
        .spawn(move || async move {
            println!("1. Creating async TCP transport with Glommio...");
            println!("   Server: {}", server_address);
            println!("   Runtime: Glommio (io_uring)");

            // Create async TCP transport with Glommio
            // Note: This will fail without a real DLMS server
            match GlommioTcpTransport::connect(server_address).await {
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
                    let mut client =
                        AsyncClientBuilder::new(transport, settings).build_with_heap(2048);

                    // Option 2: Stack-allocated buffer (compile-time size, for embedded/no_std)
                    // let mut client = AsyncClientBuilder::new(transport, settings)
                    //     .build_with_heapless::<2048>();

                    println!("   ✓ Async client created (Glommio runtime)");
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
            println!("API Usage Patterns (Glommio):");
            println!("------------------------------");
            println!("use glommio::LocalExecutorBuilder;");
            println!();
            println!("fn main() -> Result<(), std::io::Error> {{");
            println!("    let local_ex = LocalExecutorBuilder::default()");
            println!("        .spawn(|| async move {{");
            println!("            // Create async TCP transport with Glommio");
            println!(
                "            let mut transport = GlommioTcpTransport::connect(\"192.168.1.100:4059\").await?;"
            );
            println!();
            println!("            // Configure timeouts");
            println!(
                "            transport.set_read_timeout(Some(Duration::from_secs(30)));"
            );
            println!(
                "            transport.set_write_timeout(Some(Duration::from_secs(30)));"
            );
            println!();
            println!("            // Create async client");
            println!("            let settings = ClientSettings {{");
            println!("                client_address: 16,");
            println!("                server_address: 1,");
            println!("                ..Default::default()");
            println!("            }};");
            println!("            // Build client with heap buffer (2048 bytes)");
            println!("            let mut client = AsyncClientBuilder::new(transport, settings)");
            println!("                .build_with_heap(2048);");
            println!();
            println!("            // Or with heapless buffer for embedded systems:");
            println!(
                "            // let mut client = AsyncClientBuilder::new(transport, settings)"
            );
            println!("            //     .build_with_heapless::<2048>();");
            println!();
            println!("            // Connect and read data (all async with .await)");
            println!("            client.connect().await?;");
            println!(
                "            let data = client.read(class_id, obis_code, attribute_id, None).await?;"
            );
            println!("            client.disconnect().await?;");
            println!("            Ok::<(), Box<dyn std::error::Error>>(())");
            println!("        }})?;");
            println!("    local_ex.join()?;");
            println!("    Ok(())");
            println!("}}");
            println!();
            println!("Benefits of Async with Glommio:");
            println!("--------------------------------");
            println!("✓ Ultra-low latency (io_uring kernel bypass)");
            println!("✓ Thread-per-core architecture (no work stealing)");
            println!("✓ Zero-copy I/O with DMA buffers");
            println!("✓ Perfect for high-frequency trading and real-time systems");
            println!("✓ Excellent for single-threaded high-performance applications");
            println!("✓ Linux-only, requires kernel 5.8+ with io_uring");
            println!();
            println!("Buffer Allocation Options:");
            println!("---------------------------");
            println!("✓ Heap buffer: .build_with_heap(size) - flexible, runtime-determined");
            println!("✓ Stack buffer: .build_with_heapless::<SIZE>() - embedded, compile-time");
            println!("  Recommended sizes: 256 (minimal), 2048 (standard), 8192 (load profiles)");
            println!();
            println!("When to Use Glommio:");
            println!("---------------------");
            println!("✓ Linux-only deployments");
            println!("✓ Ultra-low latency requirements (<1ms)");
            println!("✓ High-throughput single-threaded applications");
            println!("✓ Direct hardware access scenarios");
            println!("✗ Cross-platform applications (use Tokio instead)");
            println!("✗ CPU-bound workloads (use Tokio with work stealing)");

            Ok::<(), std::io::Error>(())
        })
        .expect("Failed to spawn executor");

    let _ = local_ex.join().expect("Failed to join executor");
    Ok(())
}

#[cfg(not(all(feature = "async-client", feature = "transport-tcp-async", feature = "glommio")))]
fn main() {
    eprintln!(
        "This example requires the 'async-client', 'transport-tcp-async', and 'glommio' features."
    );
    eprintln!(
        "Run with: cargo run --example tcp_transport_async_glommio --features async-client,transport-tcp-async,glommio,encode,parse"
    );
    eprintln!();
    eprintln!("Platform Requirements:");
    eprintln!("  - Linux kernel 5.8+ with io_uring support");
    eprintln!("  - x86_64 or aarch64 architecture");
    std::process::exit(1);
}
