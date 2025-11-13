use fig::FigBuf;

#[test]
fn test_static_slice_basic() {
    static DATA: [i32; 5] = [1, 2, 3, 4, 5];
    let buf = FigBuf::from_static(&DATA);

    assert_eq!(buf.len(), 5);
    assert_eq!(buf[0], 1);
    assert_eq!(buf[4], 5);
    assert!(buf.is_static());
    assert_eq!(buf.ref_count(), usize::MAX);
}

#[test]
fn test_static_slice_cloning() {
    static DATA: [i32; 3] = [10, 20, 30];
    let buf1 = FigBuf::from_static(&DATA);
    let buf2 = buf1.clone();
    let buf3 = buf1.clone();

    assert!(buf1.is_static());
    assert!(buf2.is_static());
    assert!(buf3.is_static());
    assert_eq!(buf1.ref_count(), usize::MAX);
    assert_eq!(buf2.ref_count(), usize::MAX);
}

#[test]
fn test_static_slice_slicing() {
    static DATA: [i32; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    let buf = FigBuf::from_static(&DATA);

    let slice1 = buf.slice(2..8);
    assert_eq!(&*slice1, &[2, 3, 4, 5, 6, 7]);
    assert!(slice1.is_static());

    let slice2 = slice1.slice(1..4);
    assert_eq!(&*slice2, &[3, 4, 5]);
    assert!(slice2.is_static());
}

#[test]
fn test_static_string() {
    static TEXT: &str = "Hello, World!";
    let buf = FigBuf::from_static(TEXT);

    assert_eq!(&*buf, "Hello, World!");
    assert!(buf.is_static());

    let hello = buf.slice(0..5);
    assert_eq!(&*hello, "Hello");
    assert!(hello.is_static());
}

#[test]
fn test_static_empty() {
    static EMPTY: [i32; 0] = [];
    let buf = FigBuf::from_static(&EMPTY);

    assert!(buf.is_empty());
    assert!(buf.is_static());
    assert_eq!(buf.len(), 0);
}

#[test]
fn test_static_no_mut() {
    static DATA: [i32; 3] = [1, 2, 3];
    let mut buf = FigBuf::from_static(&DATA);

    // Static slices should never give mutable access
    assert!(buf.get_mut().is_none());
}

#[test]
fn test_basic_operations() {
    let buf = FigBuf::from_vec(vec![1, 2, 3, 4, 5]);
    assert_eq!(buf.len(), 5);
    assert_eq!(buf[0], 1);
    assert_eq!(buf[4], 5);
}

#[test]
fn test_slicing_operations() {
    let buf = FigBuf::from_vec(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

    let slice1 = buf.slice(2..8);
    assert_eq!(slice1.len(), 6);
    assert_eq!(&*slice1, &[2, 3, 4, 5, 6, 7]);

    let slice2 = slice1.slice(1..5);
    assert_eq!(slice2.len(), 4);
    assert_eq!(&*slice2, &[3, 4, 5, 6]);

    assert_eq!(&*buf, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
}

#[test]
fn test_reference_counting() {
    let buf = FigBuf::from_vec(vec![1, 2, 3]);
    assert_eq!(buf.ref_count(), 1);

    let buf2 = buf.clone();
    assert_eq!(buf.ref_count(), 2);
    assert_eq!(buf2.ref_count(), 2);

    let slice = buf.slice(1..3);
    assert_eq!(buf.ref_count(), 3);

    drop(buf2);
    assert_eq!(buf.ref_count(), 2);

    drop(slice);
    assert_eq!(buf.ref_count(), 1);
}

#[test]
fn test_string_operations() {
    let text = FigBuf::from_string(String::from("The quick brown fox jumps over the lazy dog"));

    let quick = text.slice(4..9);
    assert_eq!(&*quick, "quick");

    let brown = text.slice(10..15);
    assert_eq!(&*brown, "brown");

    let fox = text.slice(16..19);
    assert_eq!(&*fox, "fox");

    assert_eq!(text.ref_count(), 4);
}

#[test]
fn test_empty_slices() {
    let buf = FigBuf::from_vec(vec![1, 2, 3, 4, 5]);

    let empty = buf.slice(2..2);
    assert!(empty.is_empty());
    assert_eq!(empty.len(), 0);

    let also_empty = buf.slice(5..5);
    assert!(also_empty.is_empty());
}

#[test]
fn test_full_range_slicing() {
    let buf = FigBuf::from_vec(vec![1, 2, 3, 4, 5]);

    let full = buf.slice(..);
    assert_eq!(&*full, &*buf);

    let from_start = buf.slice(..3);
    assert_eq!(&*from_start, &[1, 2, 3]);

    let to_end = buf.slice(2..);
    assert_eq!(&*to_end, &[3, 4, 5]);
}

#[test]
fn test_deeply_nested_slicing() {
    let buf = FigBuf::from_vec((0..100).collect::<Vec<_>>());

    let level1 = buf.slice(10..90);
    let level2 = level1.slice(10..70);
    let level3 = level2.slice(10..50);
    let level4 = level3.slice(10..30);

    assert_eq!(&*level4, &(40..60).collect::<Vec<_>>()[..]);

    assert_eq!(buf.ref_count(), 5);
}

#[test]
fn test_conversions() {

    let buf1: FigBuf<[i32]> = vec![1, 2, 3].into();
    assert_eq!(&*buf1, &[1, 2, 3]);

    let buf2: FigBuf<[i32]> = (&[4, 5, 6][..]).into();
    assert_eq!(&*buf2, &[4, 5, 6]);

    let buf3: FigBuf<str> = String::from("hello").into();
    assert_eq!(&*buf3, "hello");

    let buf4: FigBuf<str> = "world".into();
    assert_eq!(&*buf4, "world");
}

#[test]
fn test_deref_coercion() {
    let buf = FigBuf::from_vec(vec![1, 2, 3, 4, 5]);

    assert!(buf.contains(&3));
    assert_eq!(buf.first(), Some(&1));
    assert_eq!(buf.last(), Some(&5));
    assert_eq!(buf.iter().sum::<i32>(), 15);
}

#[test]
fn test_multiple_slices_from_same_source() {
    let buf = FigBuf::from_vec((0..20).collect::<Vec<_>>());

    let slices: Vec<_> = (0..5)
        .map(|i| buf.slice(i * 4..(i + 1) * 4))
        .collect();

    assert_eq!(&*slices[0], &[0, 1, 2, 3]);
    assert_eq!(&*slices[1], &[4, 5, 6, 7]);
    assert_eq!(&*slices[2], &[8, 9, 10, 11]);
    assert_eq!(&*slices[3], &[12, 13, 14, 15]);
    assert_eq!(&*slices[4], &[16, 17, 18, 19]);

    assert_eq!(buf.ref_count(), 6);
}

#[test]
fn test_string_char_boundaries() {
    let text = FigBuf::from_string(String::from("Hello, 世界!"));

    let hello = text.slice(0..7);
    assert_eq!(&*hello, "Hello, ");

    let world = text.slice(7..13);
    assert_eq!(&*world, "世界");
}

#[test]
#[should_panic(expected = "slice start not at char boundary")]
fn test_string_invalid_char_boundary_start() {
    let text = FigBuf::from_string(String::from("Hello, 世界!"));

    let _ = text.slice(8..10);
}

#[test]
#[should_panic(expected = "slice end not at char boundary")]
fn test_string_invalid_char_boundary_end() {
    let text = FigBuf::from_string(String::from("Hello, 世界!"));
    let _ = text.slice(7..8);
}

#[test]
#[should_panic(expected = "slice end out of bounds")]
fn test_slice_out_of_bounds() {
    let buf = FigBuf::from_vec(vec![1, 2, 3, 4, 5]);
    let _ = buf.slice(0..10);
}

#[test]
#[should_panic(expected = "slice start must be <= end")]
fn test_slice_invalid_range() {
    let buf = FigBuf::from_vec(vec![1, 2, 3, 4, 5]);
    let _ = buf.slice(3..1);
}