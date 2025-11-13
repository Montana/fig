use fig::small::{SmallFigBuf, SmallFigStr};

#[test]
fn test_small_buf_inline_storage() {
    let buf: SmallFigBuf<32> = SmallFigBuf::from_slice(b"hello world");

    assert!(buf.is_inline());
    assert!(!buf.is_heap());
    assert_eq!(buf.len(), 11);
    assert_eq!(&*buf, b"hello world");
}

#[test]
fn test_small_buf_heap_storage() {
    let data = vec![0u8; 100];
    let buf: SmallFigBuf<32> = SmallFigBuf::from_vec(data);

    assert!(!buf.is_inline());
    assert!(buf.is_heap());
    assert_eq!(buf.len(), 100);
}

#[test]
fn test_small_buf_capacity_boundary() {

    let buf_at: SmallFigBuf<16> = SmallFigBuf::from_slice(&[0; 16]);
    assert!(buf_at.is_inline());

    let buf_over: SmallFigBuf<16> = SmallFigBuf::from_slice(&[0; 17]);
    assert!(buf_over.is_heap());
}

#[test]
fn test_small_buf_clone_inline() {
    let buf1: SmallFigBuf<32> = SmallFigBuf::from_slice(b"test data");
    let buf2 = buf1.clone();

    assert_eq!(buf1, buf2);
    assert!(buf1.is_inline());
    assert!(buf2.is_inline());

    // Both should have independent inline storage
    assert_eq!(&*buf1, b"test data");
    assert_eq!(&*buf2, b"test data");
}

#[test]
fn test_small_buf_clone_heap() {
    let buf1: SmallFigBuf<8> = SmallFigBuf::from_slice(b"this is a longer string");
    let buf2 = buf1.clone();

    assert_eq!(buf1, buf2);
    assert!(buf1.is_heap());
    assert!(buf2.is_heap());
}

#[test]
fn test_small_buf_slice_inline() {
    let buf: SmallFigBuf<32> = SmallFigBuf::from_slice(b"hello world");

    let hello = buf.slice(0..5);
    assert_eq!(&*hello, b"hello");
    assert!(hello.is_inline());

    let world = buf.slice(6..11);
    assert_eq!(&*world, b"world");
    assert!(world.is_inline());
}

#[test]
fn test_small_buf_slice_heap() {
    let data = (0..100).collect::<Vec<u8>>();
    let buf: SmallFigBuf<32> = SmallFigBuf::from_vec(data);

    let slice1 = buf.slice(0..50);
    assert_eq!(slice1.len(), 50);
    assert!(slice1.is_heap());

    let slice2 = buf.slice(25..75);
    assert_eq!(slice2.len(), 50);
    assert!(slice2.is_heap());
}

#[test]
fn test_small_buf_empty() {
    let buf: SmallFigBuf<32> = SmallFigBuf::new();

    assert!(buf.is_empty());
    assert!(buf.is_inline());
    assert_eq!(buf.len(), 0);
}

#[test]
fn test_small_buf_equality() {
    let buf1: SmallFigBuf<32> = SmallFigBuf::from_slice(b"test");
    let buf2: SmallFigBuf<32> = SmallFigBuf::from_slice(b"test");
    let buf3: SmallFigBuf<32> = SmallFigBuf::from_slice(b"different");

    assert_eq!(buf1, buf2);
    assert_ne!(buf1, buf3);
    assert_eq!(buf1, b"test");
    assert_eq!(buf1, &b"test"[..]);
}

#[test]
fn test_small_buf_from_conversions() {
    let from_vec: SmallFigBuf<32> = b"hello".to_vec().into();
    assert!(from_vec.is_inline());
    assert_eq!(&*from_vec, b"hello");

    let from_slice: SmallFigBuf<32> = (&b"world"[..]).into();
    assert!(from_slice.is_inline());
    assert_eq!(&*from_slice, b"world");

    let from_str: SmallFigBuf<32> = "test".into();
    assert!(from_str.is_inline());
    assert_eq!(&*from_str, b"test");
}

#[test]
fn test_small_buf_to_figbuf() {
    let small: SmallFigBuf<32> = SmallFigBuf::from_slice(b"test");
    let figbuf = small.to_figbuf();

    assert_eq!(&*figbuf, b"test");
}

#[test]
fn test_small_buf_into_figbuf() {
    let small: SmallFigBuf<32> = SmallFigBuf::from_slice(b"test");
    let figbuf = small.into_figbuf();

    assert_eq!(&*figbuf, b"test");
}


#[test]
fn test_small_str_inline() {
    let s: SmallFigStr<32> = SmallFigStr::from("hello world");

    assert!(s.is_inline());
    assert_eq!(s.len(), 11);
    assert_eq!(&*s, "hello world");
}

#[test]
fn test_small_str_heap() {
    let long_str = "a".repeat(100);
    let s: SmallFigStr<32> = SmallFigStr::from(long_str.as_str());

    assert!(!s.is_inline());
    assert_eq!(s.len(), 100);
}

#[test]
fn test_small_str_unicode() {
    let s: SmallFigStr<32> = SmallFigStr::from("Hello, 世界!");

    assert!(s.is_inline());
    assert_eq!(&*s, "Hello, 世界!");
}

#[test]
fn test_small_str_slice() {
    let s: SmallFigStr<32> = SmallFigStr::from("The quick brown fox");

    let the = s.slice(0..3);
    assert_eq!(&*the, "The");
    assert!(the.is_inline());

    let quick = s.slice(4..9);
    assert_eq!(&*quick, "quick");
    assert!(quick.is_inline());
}

#[test]
fn test_small_str_slice_unicode() {
    let s: SmallFigStr<32> = SmallFigStr::from("Hello, 世界!");

    let hello = s.slice(0..7);
    assert_eq!(&*hello, "Hello, ");

    let world = s.slice(7..13);
    assert_eq!(&*world, "世界");
}

#[test]
#[should_panic(expected = "slice start not at char boundary")]
fn test_small_str_invalid_boundary() {
    let s: SmallFigStr<32> = SmallFigStr::from("世界");
    let _ = s.slice(1..3); 
}

#[test]
fn test_small_str_clone() {
    let s1: SmallFigStr<32> = SmallFigStr::from("test");
    let s2 = s1.clone();

    assert_eq!(s1, s2);
    assert!(s1.is_inline());
    assert!(s2.is_inline());
}

#[test]
fn test_small_str_equality() {
    let s1: SmallFigStr<32> = SmallFigStr::from("test");
    let s2: SmallFigStr<32> = SmallFigStr::from("test");
    let s3: SmallFigStr<32> = SmallFigStr::from("different");

    assert_eq!(s1, s2);
    assert_ne!(s1, s3);
    assert_eq!(s1, "test");
    assert_eq!(s1, &"test");
}

#[test]
fn test_small_str_empty() {
    let s: SmallFigStr<32> = SmallFigStr::new();

    assert!(s.is_empty());
    assert!(s.is_inline());
    assert_eq!(s.len(), 0);
    assert_eq!(&*s, "");
}

#[test]
fn test_small_str_from_conversions() {
    let from_str: SmallFigStr<32> = "hello".into();
    assert!(from_str.is_inline());
    assert_eq!(&*from_str, "hello");

    let from_string: SmallFigStr<32> = String::from("world").into();
    assert!(from_string.is_inline());
    assert_eq!(&*from_string, "world");
}

#[test]
fn test_small_str_static() {
    static TEXT: &str = "static string";
    let s: SmallFigStr<32> = SmallFigStr::from_static(TEXT);

    assert_eq!(&*s, "static string");
}

#[test]
fn test_different_capacities() {
    let tiny: SmallFigBuf<8> = SmallFigBuf::from_slice(b"small");
    assert!(tiny.is_inline());

    let medium: SmallFigBuf<64> = SmallFigBuf::from_slice(b"this is a medium sized buffer");
    assert!(medium.is_inline());

    let large: SmallFigBuf<256> = SmallFigBuf::from_slice(&[0; 200]);
    assert!(large.is_inline());
}

#[test]
fn test_inline_capacity() {
    assert_eq!(SmallFigBuf::<16>::inline_capacity(), 16);
    assert_eq!(SmallFigBuf::<32>::inline_capacity(), 32);
    assert_eq!(SmallFigBuf::<256>::inline_capacity(), 256);
}

#[test]
fn test_zero_capacity() {

    let buf: SmallFigBuf<0> = SmallFigBuf::from_slice(b"test");
    assert!(buf.is_heap());
    assert_eq!(&*buf, b"test");
}
