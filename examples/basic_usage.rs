use fig::FigBuf;

fn main() {
    println!("=== Static Slice Example ===");
    static STATIC_DATA: [i32; 5] = [100, 200, 300, 400, 500];
    let static_buf = FigBuf::<[i32]>::from_static(&STATIC_DATA);

    println!("Static buffer: {:?}", static_buf);
    println!("Is static: {}", static_buf.is_static());
    println!("Reference count: {} (usize::MAX means static)", static_buf.ref_count());

    let static_slice = static_buf.slice(1..4);
    println!("Static slice: {:?}", static_slice);
    println!("Still static after slicing: {}", static_slice.is_static());

    static GREETING: &str = "Hello from static memory!";
    let static_str = FigBuf::<str>::from_static(GREETING);
    println!("\nStatic string: {}", static_str);
    println!("Word slice: {}", static_str.slice(0..5));

    println!("\n=== Heap-Allocated Example ===");
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let buf = FigBuf::from_vec(data);

    println!("Original buffer: {:?}", buf);
    println!("Length: {}", buf.len());
    println!("Reference count: {}", buf.ref_count());
    println!("Is static: {}", buf.is_static());

    let slice1 = buf.slice(0..5);
    let slice2 = buf.slice(5..10);

    println!("\nFirst slice: {:?}", slice1);
    println!("Second slice: {:?}", slice2);
    println!("Reference count after slicing: {}", buf.ref_count());

    let clone = buf.clone();
    println!("\nReference count after cloning: {}", buf.ref_count());

    let nested = slice1.slice(1..4);
    println!("\nNested slice (elements 1-3 from first slice): {:?}", nested);

    let text = String::from("Hello, Rust World!");
    let str_buf = FigBuf::from_string(text);

    println!("\n--- String Example ---");
    println!("Full string: {}", str_buf);

    let hello = str_buf.slice(0..5);
    let rust = str_buf.slice(7..11);
    let world = str_buf.slice(12..17);

    println!("Sliced parts: '{}', '{}', '{}'", hello, rust, world);
    println!("String reference count: {}", str_buf.ref_count());

    println!("\n--- Memory Efficiency ---");
    let large_vec: Vec<u64> = (0..1000).collect();
    let large_buf = FigBuf::from_vec(large_vec);

    let slices: Vec<_> = (0..10)
        .map(|i| large_buf.slice(i * 100..(i + 1) * 100))
        .collect();

    println!("Created 10 slices from 1000 elements");
    println!("All slices share the same allocation");
    println!("Reference count: {}", large_buf.ref_count());
    println!("First element of each slice:");
    for (i, slice) in slices.iter().enumerate() {
        println!("  Slice {}: starts with {}", i, slice[0]);
    }
}