use fig::bytes::Bytes;

#[test]
fn test_bytes_creation() {
    let bytes = Bytes::from_vec(vec![1, 2, 3, 4, 5]);
    assert_eq!(bytes.len(), 5);
    assert_eq!(&*bytes, &[1, 2, 3, 4, 5]);
}

#[test]
fn test_bytes_new() {
    let bytes = Bytes::new();
    assert!(bytes.is_empty());
    assert_eq!(bytes.len(), 0);
}

#[test]
fn test_bytes_from_static() {
    let bytes = Bytes::from_static(b"hello world");
    assert_eq!(&*bytes, b"hello world");
}

#[test]
fn test_bytes_slice() {
    let bytes = Bytes::from_vec(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

    let slice1 = bytes.slice(2..7);
    assert_eq!(&*slice1, &[2, 3, 4, 5, 6]);

    let slice2 = slice1.slice(1..4);
    assert_eq!(&*slice2, &[3, 4, 5]);
}

#[test]
fn test_bytes_split_off() {
    let mut bytes = Bytes::from_vec(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    let right = bytes.split_off(5);

    assert_eq!(&*bytes, &[0, 1, 2, 3, 4]);
    assert_eq!(&*right, &[5, 6, 7, 8, 9]);
}

#[test]
fn test_bytes_split_to() {
    let mut bytes = Bytes::from_vec(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    let left = bytes.split_to(5);

    assert_eq!(&*left, &[0, 1, 2, 3, 4]);
    assert_eq!(&*bytes, &[5, 6, 7, 8, 9]);
}

#[test]
fn test_bytes_truncate() {
    let mut bytes = Bytes::from_vec(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    bytes.truncate(5);

    assert_eq!(&*bytes, &[0, 1, 2, 3, 4]);
    assert_eq!(bytes.len(), 5);
}

#[test]
fn test_bytes_truncate_noop() {
    let mut bytes = Bytes::from_vec(vec![0, 1, 2, 3, 4]);
    bytes.truncate(10); // Truncate to length greater than current

    assert_eq!(&*bytes, &[0, 1, 2, 3, 4]);
    assert_eq!(bytes.len(), 5);
}

#[test]
fn test_bytes_clear() {
    let mut bytes = Bytes::from_vec(vec![1, 2, 3, 4, 5]);
    bytes.clear();

    assert!(bytes.is_empty());
    assert_eq!(bytes.len(), 0);
}

#[test]
fn test_bytes_clone() {
    let bytes1 = Bytes::from_vec(vec![1, 2, 3, 4, 5]);
    let bytes2 = bytes1.clone();

    assert_eq!(&*bytes1, &*bytes2);
    assert_eq!(&*bytes1, &[1, 2, 3, 4, 5]);
}

#[test]
fn test_bytes_equality() {
    let bytes1 = Bytes::from_vec(vec![1, 2, 3, 4, 5]);
    let bytes2 = Bytes::from_vec(vec![1, 2, 3, 4, 5]);
    let bytes3 = Bytes::from_vec(vec![1, 2, 3, 4, 6]);

    assert_eq!(bytes1, bytes2);
    assert_ne!(bytes1, bytes3);
}

#[test]
fn test_bytes_eq_with_slice() {
    let bytes = Bytes::from_vec(vec![1, 2, 3, 4, 5]);

    assert_eq!(bytes, vec![1, 2, 3, 4, 5]);
    assert_eq!(bytes, &[1, 2, 3, 4, 5][..]);
    assert_eq!(vec![1, 2, 3, 4, 5], bytes);
    assert_eq!(&[1, 2, 3, 4, 5][..], bytes);
}

#[test]
fn test_bytes_from_string() {
    let bytes = Bytes::from(String::from("hello world"));
    assert_eq!(&*bytes, b"hello world");
}

#[test]
fn test_bytes_from_str() {
    let bytes = Bytes::from("hello world");
    assert_eq!(&*bytes, b"hello world");
}

#[test]
fn test_bytes_to_vec() {
    let bytes = Bytes::from_vec(vec![1, 2, 3, 4, 5]);
    let vec = bytes.to_vec();

    assert_eq!(vec, vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_bytes_as_ref() {
    let bytes = Bytes::from_vec(vec![1, 2, 3, 4, 5]);
    let slice: &[u8] = bytes.as_ref();

    assert_eq!(slice, &[1, 2, 3, 4, 5]);
}

#[test]
fn test_bytes_deref() {
    let bytes = Bytes::from_vec(vec![1, 2, 3, 4, 5]);

    // Can use slice methods through Deref
    assert!(bytes.contains(&3));
    assert_eq!(bytes.first(), Some(&1));
    assert_eq!(bytes.last(), Some(&5));
}

#[test]
fn test_bytes_split_operations_preserve_data() {
    let original = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    let mut bytes = Bytes::from_vec(original.clone());

    let middle = bytes.split_to(3);
    let end = bytes.split_off(4);

    assert_eq!(&*middle, &[0, 1, 2]);
    assert_eq!(&*bytes, &[3, 4, 5, 6]);
    assert_eq!(&*end, &[7, 8, 9]);
}

#[test]
fn test_bytes_multiple_slices() {
    let bytes = Bytes::from_vec((0..100).collect());

    let slices: Vec<_> = (0..10).map(|i| bytes.slice(i * 10..(i + 1) * 10)).collect();

    for (i, slice) in slices.iter().enumerate() {
        let expected: Vec<u8> = ((i * 10)..(i + 1) * 10).map(|x| x as u8).collect();
        assert_eq!(&**slice, &expected[..]);
    }
}

#[test]
fn test_bytes_nested_operations() {
    let bytes = Bytes::from_vec((0..20).collect());

    let slice1 = bytes.slice(5..15);
    assert_eq!(&*slice1, &[5, 6, 7, 8, 9, 10, 11, 12, 13, 14]);

    let mut slice2 = slice1.slice(2..8);
    assert_eq!(&*slice2, &[7, 8, 9, 10, 11, 12]);

    let slice3 = slice2.split_off(3);
    assert_eq!(&*slice2, &[7, 8, 9]);
    assert_eq!(&*slice3, &[10, 11, 12]);
}

#[test]
fn test_bytes_debug_format() {
    let bytes = Bytes::from_vec(vec![1, 2, 3]);
    let debug_str = format!("{:?}", bytes);
    assert_eq!(debug_str, "[1, 2, 3]");
}

#[test]
fn test_bytes_empty_operations() {
    let bytes = Bytes::new();

    let slice = bytes.slice(..);
    assert!(slice.is_empty());

    let clone = bytes.clone();
    assert!(clone.is_empty());
}

#[test]
fn test_bytes_large_buffer_sharing() {
    let large: Vec<u8> = (0..10000).map(|x| (x % 256) as u8).collect();
    let bytes = Bytes::from_vec(large);

    let slices: Vec<_> = (0..100)
        .map(|i| bytes.slice(i * 100..(i + 1) * 100))
        .collect();

    for (i, slice) in slices.iter().enumerate() {
        assert_eq!(slice.len(), 100);
        assert_eq!(slice[0], (i * 100 % 256) as u8);
    }
}
