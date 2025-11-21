//! Embassy-Net TCP transport example for no_std environments.
//!
//! This example demonstrates how to use the EmbassyNetTcpTransport with
//! embassy-net for true no_std async TCP communication on embedded systems.
//!
//! # Features Required
//!
//! - `async-client`: Enables async DLMS client
//! - `transport-tcp-async`: Enables async TCP transport
//! - `embassy-net`: Enables embassy-net integration (no_std)
//! - `encode`: Enables DLMS message encoding
//! - `parse`: Enables DLMS message parsing
//!
//! # Platform Support
//!
//! This example is designed for embedded systems:
//! - STM32 microcontrollers (with embassy-stm32)
//! - ESP32 microcontrollers (with embassy-esp)
//! - nRF52 microcontrollers (with embassy-nrf)
//! - RP2040 (with embassy-rp)
//! - Any platform with embassy-net support
//!
//! # Hardware Requirements
//!
//! - Microcontroller with Ethernet or WiFi
//! - embassy-net network stack configured
//! - Access to DLMS meter on network
//!
//! # Run Example
//!
//! ```bash
//! cargo run --example tcp_transport_embassy_net_nostd \
//!   --features async-client,transport-tcp-async,embassy-net,encode,parse
//! ```
//!
//! This is a documentation example showing the pattern.
//! For actual embedded deployment, integrate with your HAL crate.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(
    feature = "async-client",
    feature = "transport-tcp-async",
    feature = "embassy-net",
    feature = "encode",
    feature = "parse"
))]
fn main() {
    println!("=================================================================");
    println!("Embassy-Net TCP Transport Example (no_std)");
    println!("=================================================================");
    println!();
    println!("Platform: Embedded (no_std) with embassy-net");
    println!("Network Stack: embassy-net");
    println!("Allocation: Heapless (stack-based buffers)");
    println!("Runtime: embassy-executor");
    println!();

    println!("What is embassy-net?");
    println!("--------------------");
    println!("✓ Async network stack for embedded systems");
    println!("✓ no_std compatible (works on bare-metal MCUs)");
    println!("✓ Zero heap allocations required");
    println!("✓ Supports TCP/UDP/DNS/DHCP");
    println!("✓ Hardware-agnostic (works with any Ethernet/WiFi driver)");
    println!("✓ Cooperative multitasking via embassy-executor");
    println!();

    println!("Complete no_std Example:");
    println!("------------------------");
    println!();
    println!("#![no_std]");
    println!("#![no_main]");
    println!();
    println!("use embassy_executor::Spawner;");
    println!("use embassy_net::{{Stack, tcp::TcpSocket, Ipv4Address}};");
    println!("use embassy_time::Duration;");
    println!("use dlms_cosem::{{");
    println!("    client::{{AsyncClientBuilder, ClientSettings}},");
    println!("    transport::tcp::{{EmbassyNetTcpTransport, EMBASSY_NET_TCP_BUFFER_SIZE}},");
    println!("    types::{{ObisCode, Data}},");
    println!("}};");
    println!();
    println!("#[embassy_executor::main]");
    println!("async fn main(spawner: Spawner) {{");
    println!();
    println!("    // =====================================================");
    println!("    // Step 1: Initialize Hardware (device-specific)");
    println!("    // =====================================================");
    println!();
    println!("    // Example for STM32 (adjust for your platform)");
    println!("    let p = embassy_stm32::init(Default::default());");
    println!();
    println!("    // =====================================================");
    println!("    // Step 2: Initialize Network Stack (device-specific)");
    println!("    // =====================================================");
    println!();
    println!("    // This is platform-specific. Example for STM32 with Ethernet:");
    println!("    //");
    println!("    // use embassy_net::{{Config, StackResources}};");
    println!("    // use embassy_stm32::eth::{{Ethernet, generic_smi::GenericSMI}};");
    println!("    //");
    println!("    // static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();");
    println!("    // let resources = RESOURCES.init(StackResources::new());");
    println!("    //");
    println!("    // // Configure network (static IP or DHCP)");
    println!("    // let config = Config::ipv4_static(embassy_net::StaticConfigV4 {{");
    println!("    //     address: Ipv4Cidr::new(Ipv4Address::new(192, 168, 1, 50), 24),");
    println!("    //     gateway: Some(Ipv4Address::new(192, 168, 1, 1)),");
    println!("    //     dns_servers: Default::default(),");
    println!("    // }});");
    println!("    //");
    println!("    // // Create Ethernet driver");
    println!("    // let eth = Ethernet::new(/* ... hardware config ... */);");
    println!("    //");
    println!("    // // Initialize stack");
    println!("    // let stack = &*make_static!(Stack::new(eth, config, resources, seed));");
    println!("    //");
    println!("    // // Spawn network task");
    println!("    // spawner.spawn(net_task(stack)).unwrap();");
    println!();
    println!("    // For this example, assume stack is initialized:");
    println!("    // let stack: &'static Stack<_> = init_network_stack(spawner, p).await;");
    println!();
    println!("    // =====================================================");
    println!("    // Step 3: Allocate TCP Buffers (static for no_std)");
    println!("    // =====================================================");
    println!();
    println!("    static mut RX_BUFFER: [u8; EMBASSY_NET_TCP_BUFFER_SIZE] = ");
    println!("        [0; EMBASSY_NET_TCP_BUFFER_SIZE];");
    println!("    static mut TX_BUFFER: [u8; EMBASSY_NET_TCP_BUFFER_SIZE] = ");
    println!("        [0; EMBASSY_NET_TCP_BUFFER_SIZE];");
    println!();
    println!("    // =====================================================");
    println!("    // Step 4: Create and Connect TCP Socket");
    println!("    // =====================================================");
    println!();
    println!("    let mut socket = TcpSocket::new(");
    println!("        stack,");
    println!("        unsafe {{ &mut RX_BUFFER }},");
    println!("        unsafe {{ &mut TX_BUFFER }}");
    println!("    );");
    println!();
    println!("    // Connect to DLMS meter");
    println!("    let remote_endpoint = (Ipv4Address::new(192, 168, 1, 100), 4059);");
    println!("    socket.connect(remote_endpoint).await.unwrap();");
    println!();
    println!("    // =====================================================");
    println!("    // Step 5: Create Embassy-Net Transport");
    println!("    // =====================================================");
    println!();
    println!("    let mut transport = EmbassyNetTcpTransport::new(socket);");
    println!("    transport.set_read_timeout(Some(Duration::from_secs(60)));");
    println!("    transport.set_write_timeout(Some(Duration::from_secs(30)));");
    println!();
    println!("    // =====================================================");
    println!("    // Step 6: Configure DLMS Client Settings");
    println!("    // =====================================================");
    println!();
    println!("    let settings = ClientSettings {{");
    println!("        client_address: 16,");
    println!("        server_address: 1,");
    println!("        ..Default::default()");
    println!("    }};");
    println!();
    println!("    // =====================================================");
    println!("    // Step 7: Create Async DLMS Client (heapless buffer)");
    println!("    // =====================================================");
    println!();
    println!("    // IMPORTANT: Use .build_with_heapless() for no_std");
    println!("    // This allocates the buffer on the stack, not the heap");
    println!("    let mut client = AsyncClientBuilder::new(transport, settings)");
    println!("        .build_with_heapless::<2048>();  // 2KB stack-allocated buffer");
    println!();
    println!("    // =====================================================");
    println!("    // Step 8: Connect to DLMS Meter");
    println!("    // =====================================================");
    println!();
    println!("    client.connect().await.unwrap();");
    println!();
    println!("    // =====================================================");
    println!("    // Step 9: Read Meter Data");
    println!("    // =====================================================");
    println!();
    println!("    // Example: Read device ID (OBIS 0-0:96.1.0.255)");
    println!("    let obis = ObisCode::new(0, 0, 96, 1, 0, 255);");
    println!("    let data = client.read(1, obis, 2, None).await.unwrap();");
    println!();
    println!("    match data {{");
    println!("        Data::OctetString(bytes) => {{");
    println!("            // Process device ID...");
    println!("        }}");
    println!("        _ => {{}}");
    println!("    }}");
    println!();
    println!("    // =====================================================");
    println!("    // Step 10: Disconnect");
    println!("    // =====================================================");
    println!();
    println!("    client.disconnect().await.unwrap();");
    println!("}}");
    println!();

    println!("=================================================================");
    println!("Key Differences: std vs no_std");
    println!("=================================================================");
    println!();
    println!("┌─────────────────────────┬──────────────────────┬─────────────────────┐");
    println!("│ Aspect                  │ std (Embassy)        │ no_std (Embassy-Net)│");
    println!("├─────────────────────────┼──────────────────────┼─────────────────────┤");
    println!("│ Network Stack           │ std::net::TcpStream  │ embassy-net Stack   │");
    println!("│ Buffer Allocation       │ Heap or Stack        │ Stack only          │");
    println!("│ Client Builder          │ .build_with_heap()   │ .build_with_heapless│");
    println!("│ Standard Library        │ Required             │ Not required        │");
    println!("│ Platform                │ Desktop/Server       │ Embedded MCUs       │");
    println!("│ Memory Requirements     │ ~1MB+ RAM            │ ~10KB RAM           │");
    println!("└─────────────────────────┴──────────────────────┴─────────────────────┘");
    println!();

    println!("=================================================================");
    println!("Memory Requirements (no_std)");
    println!("=================================================================");
    println!();
    println!("TCP RX buffer:           2048 bytes  (static)");
    println!("TCP TX buffer:           2048 bytes  (static)");
    println!("DLMS client buffer:      2048 bytes  (heapless)");
    println!("Stack usage (futures):   ~4KB        (approximate)");
    println!("embassy-net overhead:    ~2KB        (stack state)");
    println!("────────────────────────────────────────────────");
    println!("Total RAM required:      ~12KB       (no heap)");
    println!();

    println!("=================================================================");
    println!("Supported Embedded Platforms");
    println!("=================================================================");
    println!();
    println!("✓ STM32 Family (embassy-stm32)");
    println!("  - ARM Cortex-M4/M7 with Ethernet");
    println!("  - Example: STM32F4, STM32F7, STM32H7");
    println!();
    println!("✓ ESP32 Family (embassy-esp)");
    println!("  - Xtensa and RISC-V variants");
    println!("  - Example: ESP32, ESP32-C3, ESP32-S3");
    println!();
    println!("✓ Nordic nRF52 (embassy-nrf)");
    println!("  - ARM Cortex-M4 with WiFi module");
    println!("  - Example: nRF52840 + ESP-AT");
    println!();
    println!("✓ Raspberry Pi Pico (embassy-rp)");
    println!("  - RP2040 with Ethernet module");
    println!("  - Example: Pico W with WizNet W5500");
    println!();

    println!("=================================================================");
    println!("When to Use embassy-net");
    println!("=================================================================");
    println!();
    println!("✅ RECOMMENDED FOR:");
    println!("  • Embedded systems with <1MB RAM");
    println!("  • Battery-powered IoT devices (low power)");
    println!("  • Real-time meter reading (deterministic)");
    println!("  • no_std environments (bare-metal)");
    println!("  • Microcontrollers (ARM Cortex-M, RISC-V)");
    println!("  • Smart meters with embedded DLMS client");
    println!();
    println!("❌ NOT RECOMMENDED FOR:");
    println!("  • High-throughput servers (use Tokio)");
    println!("  • Desktop applications (use Tokio/Smol)");
    println!("  • Systems with >100MB RAM (overkill)");
    println!("  • CPU-bound workloads (use Tokio work-stealing)");
    println!();

    println!("=================================================================");
    println!("Additional Resources");
    println!("=================================================================");
    println!();
    println!("Embassy Documentation:");
    println!("  https://embassy.dev/book/");
    println!();
    println!("embassy-net Examples:");
    println!("  https://github.com/embassy-rs/embassy/tree/main/examples");
    println!();
    println!("DLMS/COSEM Specification:");
    println!("  IEC 62056 (DLMS/COSEM)");
    println!("  Green Book Edition 12");
    println!();
    println!("Platform-Specific Setup:");
    println!("  STM32:  https://github.com/embassy-rs/embassy/tree/main/examples/stm32");
    println!("  ESP32:  https://github.com/esp-rs/esp-hal");
    println!("  nRF52:  https://github.com/embassy-rs/embassy/tree/main/examples/nrf52840");
    println!("  RP2040: https://github.com/embassy-rs/embassy/tree/main/examples/rp");
    println!();

    println!("=================================================================");
    println!("Example Complete!");
    println!("=================================================================");
    println!();
    println!("This example shows the code pattern for using embassy-net with");
    println!("dlms-cosem-rs in a no_std embedded environment.");
    println!();
    println!("For actual deployment, integrate with your platform's HAL crate");
    println!("and configure the network stack according to your hardware.");
    println!();
}

#[cfg(not(all(
    feature = "async-client",
    feature = "transport-tcp-async",
    feature = "embassy-net",
    feature = "encode",
    feature = "parse"
)))]
fn main() {
    eprintln!("=================================================================");
    eprintln!("Embassy-Net TCP Transport Example");
    eprintln!("=================================================================");
    eprintln!();
    eprintln!("This example requires the following features:");
    eprintln!("  - async-client");
    eprintln!("  - transport-tcp-async");
    eprintln!("  - embassy-net");
    eprintln!("  - encode");
    eprintln!("  - parse");
    eprintln!();
    eprintln!("Run with:");
    eprintln!("  cargo run --example tcp_transport_embassy_net_nostd \\");
    eprintln!("    --features async-client,transport-tcp-async,embassy-net,encode,parse");
    eprintln!();
    eprintln!("Platform Support:");
    eprintln!("  - no_std environments: Uses embassy-net Stack (true no_std)");
    eprintln!("  - Embedded systems: STM32, ESP32, nRF52, RP2040, etc.");
    eprintln!();
    std::process::exit(1);
}
