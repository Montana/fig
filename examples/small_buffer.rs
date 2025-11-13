use fig::small::{SmallFigBuf, SmallFigStr};

fn main() {
    println!("=== Small Buffer Optimization Demo ===\n");

    println!("--- Inline Storage ---");
    let small: SmallFigBuf<32> = SmallFigBuf::from_slice(b"hello world");

    println!("Data: {:?}", std::str::from_utf8(&small).unwrap());
    println!("Length: {}", small.len());
    println!("Is inline: {}", small.is_inline());
    println!("Is heap: {}", small.is_heap());
    println!("Inline capacity: {}", SmallFigBuf::<32>::inline_capacity());

    let cloned = small.clone();
    println!("\nAfter cloning:");
    println!("Original is inline: {}", small.is_inline());
    println!("Clone is inline: {}", cloned.is_inline());

    let slice = small.slice(0..5);
    println!("\nSliced 'hello':");
    println!("Data: {:?}", std::str::from_utf8(&slice).unwrap());
    println!("Still inline: {}", slice.is_inline());

    println!("\n--- Heap Storage (Automatic Fallback) ---");
    let large_data = vec![0u8; 100];
    let large: SmallFigBuf<32> = SmallFigBuf::from_vec(large_data);

    println!("Data length: {}", large.len());
    println!("Is inline: {}", large.is_inline());
    println!("Is heap: {}", large.is_heap());

    println!("\n--- Capacity Boundary ---");
    let exactly_32 = vec![1u8; 32];
    let at_capacity: SmallFigBuf<32> = SmallFigBuf::from_vec(exactly_32);
    println!("Exactly 32 bytes - is inline: {}", at_capacity.is_inline());

    let one_more = vec![1u8; 33];
    let over_capacity: SmallFigBuf<32> = SmallFigBuf::from_vec(one_more);
    println!("33 bytes (one over) - is inline: {}", over_capacity.is_inline());

    println!("\n=== SmallFigStr Examples ===\n");

    println!("--- Short Strings (Inline) ---");
    let short_str: SmallFigStr<32> = SmallFigStr::from("Rust");
    println!("String: '{}'", short_str);
    println!("Is inline: {}", short_str.is_inline());

    let emoji: SmallFigStr<32> = SmallFigStr::from("Hello 👋 World 🌍");
    println!("\nUnicode: '{}'", emoji);
    println!("Length (bytes): {}", emoji.len());
    println!("Is inline: {}", emoji.is_inline());

    let text: SmallFigStr<64> = SmallFigStr::from("The quick brown fox jumps");
    let quick = text.slice(4..9);
    println!("\nOriginal: '{}'", text);
    println!("Sliced: '{}'", quick);
    println!("Slice is inline: {}", quick.is_inline());

    println!("\n--- Long Strings (Heap) ---");
    let long = "a".repeat(100);
    let long_str: SmallFigStr<32> = SmallFigStr::from(&long);
    println!("String length: {}", long_str.len());
    println!("Is inline: {}", long_str.is_inline());

    println!("\n=== Different Inline Capacities ===\n");

    let tiny: SmallFigBuf<8> = SmallFigBuf::from_slice(b"small");
    println!("SmallFigBuf<8> with 'small' (5 bytes):");
    println!("  Is inline: {}", tiny.is_inline());

    let tiny_over: SmallFigBuf<8> = SmallFigBuf::from_slice(b"toolarge!");
    println!("SmallFigBuf<8> with 'toolarge!' (9 bytes):");
    println!("  Is inline: {}", tiny_over.is_inline());

    let medium: SmallFigBuf<64> = SmallFigBuf::from_slice(b"This fits comfortably in 64 bytes");
    println!("SmallFigBuf<64> with 34 byte string:");
    println!("  Is inline: {}", medium.is_inline());

    println!("\n=== Practical Use Case: Config Strings ===\n");

    let configs: Vec<SmallFigStr<32>> = vec![
        SmallFigStr::from("localhost"),
        SmallFigStr::from("127.0.0.1"),
        SmallFigStr::from("production"),
        SmallFigStr::from("debug"),
        SmallFigStr::from("/var/log/app.log"),
    ];

    println!("Configuration strings stored:");
    for (i, config) in configs.iter().enumerate() {
        println!(
            "  {}: '{}' (len={}, inline={})",
            i,
            config,
            config.len(),
            config.is_inline()
        );
    }

    println!("\n=== Memory Efficiency ===\n");
    println!(
        "Size of SmallFigBuf<32>: {} bytes",
        std::mem::size_of::<SmallFigBuf<32>>()
    );
    println!(
        "Size of SmallFigBuf<64>: {} bytes",
        std::mem::size_of::<SmallFigBuf<64>>()
    );
    println!(
        "Size of Vec<u8>: {} bytes (just the pointer, not data)",
        std::mem::size_of::<Vec<u8>>()
    );

    println!(
        "\nWith SmallFigBuf<32>, strings up to 32 bytes require:");
    println!("  - No heap allocation");
    println!("  - No pointer chasing");
    println!("  - Better cache locality");
    println!("  - Faster cloning (memcpy vs Arc clone)");
}