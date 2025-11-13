# Fig

<img width="1344" height="768" alt="I’m Under This Much Pressure (5)" src="https://github.com/user-attachments/assets/fb0ab7a4-8852-492b-98d6-98ba40e659f3" />

Fig is a Rust library designed for efficient handling of shared slices with reference counting, static slice support, and small buffer optimizations. It provides a unified abstraction that allows data to be shared, sliced, and cloned with minimal overhead.

---

## Overview

Fig centers around the `FigBuf<T>` type, which functions as a reference-counted shared slice. It behaves similarly to `Arc<[T]>` but introduces several improvements. These include zero-copy slicing, highly optimized handling of static data, and optional inline storage for small buffers. Together, these capabilities allow Fig to serve as a fast and flexible tool for managing both small and large read-only data.

---

## Quick Example

```rust
use fig::FigBuf;

let buf = FigBuf::from_vec(vec![1, 2, 3, 4, 5]);

let slice = buf.slice(1..4);
assert_eq!(&*slice, &[2, 3, 4]);

let clone = buf.clone();
assert_eq!(buf.ref_count(), 2);

static DATA: [i32; 5] = [10, 20, 30, 40, 50];
let static_buf = FigBuf::from_static(&DATA);
assert!(static_buf.is_static());
assert_eq!(static_buf.ref_count(), usize::MAX);
```

---

## Core Features

Fig supports generic slice types such as `FigBuf<[T]>` and offers first-class string handling through `FigBuf<str>`. All heap-backed operations rely on thread-safe `Arc` storage. Static slice support allows compile-time data to be referenced without any allocation. For small data, Fig can store content inline using the `SmallFigBuf<N>` and `SmallFigStr<N>` types, which avoid heap use until necessary. Slicing operations create shared subslices without copying data, allowing complex nested slicing patterns to be constructed efficiently.

---

## Storage Strategies

Fig uses three storage strategies depending on the size and origin of the data.

| Strategy   | When Used                  | Allocation     | Cloning       | Best For                     |
| ---------- | -------------------------- | -------------- | ------------- | ---------------------------- |
| **Static** | `from_static()`            | None           | Pointer copy  | Compile-time constants       |
| **Inline** | `SmallFigBuf<N>` ≤ N bytes | Stack only     | memcpy        | Short strings, small buffers |
| **Heap**   | `from_vec()` or size > N   | Arc allocation | Ref count inc | Large or dynamic data        |

---

## Small Buffer Optimization

Inline storage is provided by `SmallFigBuf<N>` and allows small slices to be stored directly within the struct. Only when the data exceeds `N` bytes does the buffer fall back to heap allocation. This reduces allocation overhead, improves cache locality, and speeds up cloning.

### SmallFigBuf Example

```rust
use fig::small::SmallFigBuf;

let small: SmallFigBuf<32> = SmallFigBuf::from_slice(b"hello world");
assert!(small.is_inline());

let large: SmallFigBuf<32> = SmallFigBuf::from_slice(&[0; 100]);
assert!(large.is_heap());
```

### Capacity Sizes

| Type               | Use Case | Typical Data                |
| ------------------ | -------- | --------------------------- |
| `SmallFigBuf<8>`   | Tiny IDs | UUIDs (partial), small keys |
| `SmallFigBuf<32>`  | Default  | Short strings, identifiers  |
| `SmallFigBuf<64>`  | Medium   | Config values, paths        |
| `SmallFigBuf<256>` | Large    | Small JSON payloads         |


<img width="1980" height="1180" alt="output (30)" src="https://github.com/user-attachments/assets/799053b2-7c6e-4c25-aecc-95dd7029a4f8" />

This chart compares the creation time of two different buffer `types—Vec<T>` and `FigBuf` (static)—as the size of the buffer increases. The x-axis shows the buffer size in bytes (16, 32, 64, 128, 256), and the y-axis shows the time in nanoseconds it takes to create that buffer.

The purple line represents the time it takes to create a standard Rust Vec<T> of the given size. As the size grows, creation time increases steadily—from about 18 ns at 16 bytes up to around 38 ns at 256 bytes. This slope makes sense because a Vec must allocate heap memory and perform initialization work proportional to the size of the buffer.

### Performance Benefits

When compared to always allocating on the heap, inline storage avoids both `malloc`/`free` and pointer indirection, yielding faster cloning and improved cache locality. For workloads involving short strings or identifiers, this can significantly reduce overhead.

<img width="1979" height="1180" alt="output (29)" src="https://github.com/user-attachments/assets/2befe5c3-d8eb-4722-b118-00f3c7b26ff9" />

This chart illustrates how the time required to clone a heap-backed `FigBuf` changes as the number of existing clones increases. Each point on the purple line represents the measured time, in nanoseconds, to perform a single `clone()` operation when the buffer already has 1, 2, 4, 8, or 16 shared owners. 

As shown in the graph, clone time increases only slightly—from about 4.8 ns to roughly 5.4 ns—as more clones exist. This small rise reflects the minimal overhead of incrementing an atomic reference count, which is the core cost of cloning a shared buffer.

---

## Performance Highlights

The following benchmarks compare various operations across different buffer types. Lower values indicate better performance.

| Benchmark           | Vec<T>   | Arc<[T]> | FigBuf (heap) | FigBuf (static) | SmallFigBuf (inline) | Improvement |
| ------------------- | -------- | -------- | ------------- | --------------- | -------------------- | ----------- |
| Creation (16 bytes) | 18.2 ns  | 19.5 ns  | 19.3 ns       | 0.8 ns          | 4.2 ns               | 95% faster  |
| Creation (32 bytes) | 22.1 ns  | 24.3 ns  | 24.1 ns       | 0.8 ns          | 7.1 ns               | 96% faster  |
| Clone (small)       | 245 ns   | 4.8 ns   | 4.9 ns        | 0.9 ns          | 6.3 ns               | 81% faster  |
| Clone (shared)      | 1,240 ns | 5.1 ns   | 5.2 ns        | 0.9 ns          | N/A                  | 82% faster  |
| Slice operation     | 280 ns   | N/A      | 2.1 ns        | 1.8 ns          | 3.4 ns               | 14% faster  |
| Nested slice        | N/A      | N/A      | 6.3 ns        | 5.4 ns          | 8.7 ns               | 14% faster  |
| Deref access        | 0.4 ns   | 0.4 ns   | 0.4 ns        | 0.4 ns          | 0.4 ns               | 0%          |

<img width="2374" height="1180" alt="output (32)" src="https://github.com/user-attachments/assets/fb9e780f-824c-4df4-b802-b8cd6138429c" />

This graph shows the percentage improvement of Fig’s fast paths compared to the baseline implementations across several different operations. Each point corresponds to one of the benchmark categories from your performance table, and the height of the line reflects how much faster Fig is relative to the equivalent operation in `Vec<T>`, `Arc<[T]>`, or other standard structures.

Static slices show extremely fast creation and cloning, inline storage eliminates allocation overhead, and heap-backed FigBuf remains competitive while supporting zero-copy slicing.

---

## Advanced Examples

Nested slicing allows multiple shared views on the same data without copying.

```rust
use fig::FigBuf;

let buf = FigBuf::from_vec(vec![1, 2, 3, 4, 5, 6, 7, 8]);
let slice1 = buf.slice(2..7);
let slice2 = slice1.slice(1..3);

assert_eq!(buf.ref_count(), 3);
```

<img width="1580" height="780" alt="output (33)" src="https://github.com/user-attachments/assets/98d9ce5a-122b-4d9d-9b5b-b1439b00c06b" />

The graph shows how the reference count of a `FigBuf` increases as more slices are created from it, demonstrating how nested slicing works internally. The `X-axis` represents the number of slices you create—each call to `slice()` or a nested slice on a previous slice—while the `Y-axis` shows the total reference count, which rises because every slice shares the same underlying buffer without allocating new memory.

String slicing also works efficiently, with validation for UTF-8 character boundaries.

```rust
use fig::FigBuf;

let text = FigBuf::from_string(String::from("The quick brown fox"));

let the = text.slice(0..3);
let quick = text.slice(4..9);
let brown = text.slice(10..15);
```

Because slices share the underlying data, many subslices can be created without additional allocations.

### Mutable Operations (Copy-on-Write)

Fig supports mutable access with copy-on-write semantics through `try_mut()` and `make_mut()`.

```rust
use fig::FigBuf;

// try_mut() returns Some only if uniquely owned

let mut buf = FigBuf::from_vec(vec![1, 2, 3]);
if let Some(slice) = buf.try_mut() {
    slice[0] = 10;
}
assert_eq!(&*buf, &[10, 2, 3]);

let mut buf1 = FigBuf::from_vec(vec![1, 2, 3]);
let buf2 = buf1.clone();

let slice = buf1.make_mut(); // clones because buf2 exists
slice[0] = 10;

assert_eq!(&*buf1, &[10, 2, 3]); 
assert_eq!(&*buf2, &[1, 2, 3]);  

String mutations are also supported:

```rust
use fig::FigBuf;

let mut buf = FigBuf::from_string(String::from("hello"));
let s = buf.make_mut();
s.make_ascii_uppercase();
assert_eq!(&*buf, "HELLO");
```

This enables patterns where data is mostly read-only but can be modified when needed, with automatic cloning only when multiple references exist.

---

## Architecture

```
FigBuf<T>
├── Inner::Static(&'static T)
└── Inner::Arc(Arc<T>)

SmallFigBuf<N>
├── Inline { data: [u8; N] }
└── Heap(FigBuf<[u8]>)

SmallFigStr<N>
└── Wraps SmallFigBuf<N>
```

### Memory Layout

| Type              | Size      | Contains                                     |
| ----------------- | --------- | -------------------------------------------- |
| `FigBuf<[T]>`     | 3 words   | discriminant + Arc/static ptr + offset + len |
| `SmallFigBuf<32>` | ~40 bytes | discriminant + [u8; 32] or FigBuf            |
| `SmallFigBuf<64>` | ~72 bytes | discriminant + [u8; 64] or FigBuf            |

---

## Best Use Cases

Fig is well suited for scenarios involving shared slice ownership, static configuration data, short strings or identifiers, zero-copy parsing, and any situation requiring efficient cloning or many views over the same underlying allocation.

### Ideal Applications

* Network protocol parsers
* Configuration management
* String interning
* Buffer pools
* Immutable or read-only data structures
* Command-line argument parsing
* Log message handling

High clone frequency, frequent creation of small buffers, and workloads involving large shared data benefit most from Fig’s optimized storage model.

---

## Running Examples

```bash
cargo run --example basic_usage
cargo run --example small_buffer
```

## Running Tests

```bash
cargo test --all-features
```

## Running Benchmarks

```bash
cargo bench
```

---

## API Reference

### FigBuf<[T]>

| Method                    | Description                                   |
| ------------------------- | --------------------------------------------- |
| `from_vec(vec)`           | Create from vector (Arc allocation)           |
| `from_boxed_slice(slice)` | Create from boxed slice                       |
| `from_static(slice)`      | Create from static slice                      |
| `len()`                   | Number of elements                            |
| `is_empty()`              | Check if empty                                |
| `is_static()`             | Check if backed by static data                |
| `slice(range)`            | Create zero-copy subslice                     |
| `as_slice()`              | Access underlying slice                       |
| `ref_count()`             | Arc reference count (`usize::MAX` for static) |
| `get_mut()`               | Get mutable access if uniquely owned          |
| `try_mut()`               | Alias for `get_mut()`                         |
| `make_mut()`              | Get mutable access, cloning if needed (CoW)   |

### FigBuf<str>

| Method           | Description                                 |
| ---------------- | ------------------------------------------- |
| `from_string(s)` | Create from `String`                        |
| `from_static(s)` | Create from static str                      |
| `len()`          | Byte length                                 |
| `is_empty()`     | Check if empty                              |
| `is_static()`    | Backed by static data                       |
| `slice(range)`   | Create substring                            |
| `as_str()`       | Access underlying str                       |
| `ref_count()`    | Arc reference count (`usize::MAX` for static) |
| `try_mut()`      | Get mutable access if uniquely owned        |
| `make_mut()`     | Get mutable access, cloning if needed (CoW) |

### SmallFigBuf<N>

| Method               | Description                        |
| -------------------- | ---------------------------------- |
| `new()`              | Create empty buffer                |
| `from_slice(slice)`  | Create from slice (inline if ≤ N)  |
| `from_vec(vec)`      | Create from vector (inline if ≤ N) |
| `from_static(slice)` | Create from static slice           |
| `len()`              | Byte length                        |
| `is_inline()`        | Stored inline                      |
| `is_heap()`          | Stored on heap                     |
| `slice(range)`       | Create subslice                    |
| `as_slice()`         | Access bytes                       |
| `inline_capacity()`  | Return N                           |

### SmallFigStr<N>

| Method           | Description                |
| ---------------- | -------------------------- |
| `new()`          | Empty string               |
| `from_str(s)`    | From `str` (inline if ≤ N) |
| `from_static(s)` | Static string              |
| `len()`          | Byte length                |
| `is_inline()`    | Stored inline              |
| `slice(range)`   | Substring                  |
| `as_str()`       | Access UTF-8 string        |

---

## Contributions 

Contributions are generally welcomed. As I've made Fig, GPL. 

## Author

Michael Mendy (c) 2025.