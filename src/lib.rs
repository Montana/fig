use std::borrow::Borrow;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::io::{self, Read, Write};
use std::ops::{Deref, RangeBounds};
use std::sync::Arc;

pub mod bytes;
pub mod small;

enum Inner<T: ?Sized + 'static> {
    Static(&'static T),

    Arc(Arc<T>),
}

impl<T: ?Sized + 'static> Clone for Inner<T> {
    fn clone(&self) -> Self {
        match self {
            Inner::Static(s) => Inner::Static(s),
            Inner::Arc(arc) => Inner::Arc(Arc::clone(arc)),
        }
    }
}

pub struct FigBuf<T: ?Sized + 'static> {
    inner: Inner<T>,
    offset: usize,
    len: usize,
}

impl<T: 'static> FigBuf<[T]> {
    pub fn from_vec(vec: Vec<T>) -> Self {
        let len = vec.len();
        Self {
            inner: Inner::Arc(Arc::from(vec.into_boxed_slice())),
            offset: 0,
            len,
        }
    }

    pub fn from_boxed_slice(slice: Box<[T]>) -> Self {
        let len = slice.len();
        Self {
            inner: Inner::Arc(Arc::from(slice)),
            offset: 0,
            len,
        }
    }

    pub fn from_static(slice: &'static [T]) -> Self {
        Self {
            inner: Inner::Static(slice),
            offset: 0,
            len: slice.len(),
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn slice(&self, range: impl RangeBounds<usize>) -> Self {
        use std::ops::Bound;

        let start = match range.start_bound() {
            Bound::Included(&n) => n,
            Bound::Excluded(&n) => n + 1,
            Bound::Unbounded => 0,
        };

        let end = match range.end_bound() {
            Bound::Included(&n) => n + 1,
            Bound::Excluded(&n) => n,
            Bound::Unbounded => self.len,
        };

        assert!(start <= end, "slice start must be <= end");
        assert!(end <= self.len, "slice end out of bounds");

        Self {
            inner: self.inner.clone(),
            offset: self.offset + start,
            len: end - start,
        }
    }

    pub fn as_slice(&self) -> &[T] {
        let full_slice = match &self.inner {
            Inner::Static(s) => s,
            Inner::Arc(arc) => &**arc,
        };
        &full_slice[self.offset..self.offset + self.len]
    }

    pub fn get_mut(&mut self) -> Option<&mut [T]> {
        match &mut self.inner {
            Inner::Static(_) => None,
            Inner::Arc(arc) => {
                Arc::get_mut(arc).map(|slice| &mut slice[self.offset..self.offset + self.len])
            }
        }
    }

    pub fn try_mut(&mut self) -> Option<&mut [T]> {
        self.get_mut()
    }

    pub fn make_mut(&mut self) -> &mut [T]
    where
        T: Clone,
    {
        let needs_clone = match &self.inner {
            Inner::Static(_) => true,
            Inner::Arc(arc) => {
                self.offset != 0 || self.len != arc.len() || Arc::strong_count(arc) > 1
            }
        };

        if needs_clone {
            let cloned_data = self.as_slice().to_vec();
            *self = Self::from_vec(cloned_data);
        }

        self.get_mut().expect("should have unique ownership")
    }

    pub fn ref_count(&self) -> usize {
        match &self.inner {
            Inner::Static(_) => usize::MAX,
            Inner::Arc(arc) => Arc::strong_count(arc),
        }
    }

    pub fn is_static(&self) -> bool {
        matches!(&self.inner, Inner::Static(_))
    }
}

impl FigBuf<str> {
    pub fn from_string(s: String) -> Self {
        let bytes = FigBuf::from_vec(s.into_bytes());
        Self {
            inner: match bytes.inner {
                Inner::Arc(arc) => {
                    Inner::Arc(unsafe { Arc::from_raw(Arc::into_raw(arc) as *const str) })
                }
                Inner::Static(_) => unreachable!("from_vec never returns Static"),
            },
            offset: 0,
            len: bytes.len,
        }
    }

    pub fn from_static(s: &'static str) -> Self {
        Self {
            inner: Inner::Static(s),
            offset: 0,
            len: s.len(),
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn slice(&self, range: impl RangeBounds<usize>) -> Self {
        use std::ops::Bound;

        let start = match range.start_bound() {
            Bound::Included(&n) => n,
            Bound::Excluded(&n) => n + 1,
            Bound::Unbounded => 0,
        };

        let end = match range.end_bound() {
            Bound::Included(&n) => n + 1,
            Bound::Excluded(&n) => n,
            Bound::Unbounded => self.len,
        };

        assert!(start <= end, "slice start must be <= end");
        assert!(end <= self.len, "slice end out of bounds");
        assert!(
            self.as_str().is_char_boundary(start),
            "slice start not at char boundary"
        );
        assert!(
            self.as_str().is_char_boundary(end),
            "slice end not at char boundary"
        );

        Self {
            inner: self.inner.clone(),
            offset: self.offset + start,
            len: end - start,
        }
    }

    pub fn as_str(&self) -> &str {
        let full_str = match &self.inner {
            Inner::Static(s) => s,
            Inner::Arc(arc) => &**arc,
        };
        &full_str[self.offset..self.offset + self.len]
    }

    pub fn ref_count(&self) -> usize {
        match &self.inner {
            Inner::Static(_) => usize::MAX,
            Inner::Arc(arc) => Arc::strong_count(arc),
        }
    }

    pub fn try_mut(&mut self) -> Option<&mut str> {
        match &mut self.inner {
            Inner::Static(_) => None,
            Inner::Arc(arc) => Arc::get_mut(arc).map(|s| unsafe {
                let bytes = s.as_bytes_mut();
                let slice = &mut bytes[self.offset..self.offset + self.len];
                std::str::from_utf8_unchecked_mut(slice)
            }),
        }
    }

    pub fn make_mut(&mut self) -> &mut str {
        let needs_clone = match &self.inner {
            Inner::Static(_) => true,
            Inner::Arc(arc) => {
                self.offset != 0 || self.len != arc.len() || Arc::strong_count(arc) > 1
            }
        };

        if needs_clone {
            let cloned_data = self.as_str().to_string();
            *self = Self::from_string(cloned_data);
        }

        self.try_mut().expect("should have unique ownership")
    }

    pub fn is_static(&self) -> bool {
        matches!(&self.inner, Inner::Static(_))
    }
}

impl<T: 'static> Clone for FigBuf<[T]> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            offset: self.offset,
            len: self.len,
        }
    }
}

impl Clone for FigBuf<str> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            offset: self.offset,
            len: self.len,
        }
    }
}

impl<T: 'static> Deref for FigBuf<[T]> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl Deref for FigBuf<str> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl<T: 'static> AsRef<[T]> for FigBuf<[T]> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl AsRef<str> for FigBuf<str> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<T: fmt::Debug + 'static> fmt::Debug for FigBuf<[T]> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_slice(), f)
    }
}

impl fmt::Debug for FigBuf<str> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl<T: fmt::Display + 'static> fmt::Display for FigBuf<[T]> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for (i, item) in self.as_slice().iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", item)?;
        }
        write!(f, "]")
    }
}

impl fmt::Display for FigBuf<str> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

impl<T: 'static> From<Vec<T>> for FigBuf<[T]> {
    fn from(vec: Vec<T>) -> Self {
        Self::from_vec(vec)
    }
}

impl<T: 'static> From<Box<[T]>> for FigBuf<[T]> {
    fn from(slice: Box<[T]>) -> Self {
        Self::from_boxed_slice(slice)
    }
}

impl From<String> for FigBuf<str> {
    fn from(s: String) -> Self {
        Self::from_string(s)
    }
}

impl<'a, T: Clone + 'static> From<&'a [T]> for FigBuf<[T]> {
    fn from(slice: &'a [T]) -> Self {
        Self::from_vec(slice.to_vec())
    }
}

impl<'a> From<&'a str> for FigBuf<str> {
    fn from(s: &'a str) -> Self {
        Self::from_string(s.to_string())
    }
}

impl<T: Hash + 'static> Hash for FigBuf<[T]> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_slice().hash(state);
    }
}

impl Hash for FigBuf<str> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl<T: PartialEq + 'static> PartialEq for FigBuf<[T]> {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T: Eq + 'static> Eq for FigBuf<[T]> {}

impl PartialEq for FigBuf<str> {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for FigBuf<str> {}

impl<T: 'static> Borrow<[T]> for FigBuf<[T]> {
    fn borrow(&self) -> &[T] {
        self.as_slice()
    }
}

impl Borrow<str> for FigBuf<str> {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl Read for FigBuf<[u8]> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let data = self.as_slice();
        let len = std::cmp::min(buf.len(), data.len());
        buf[..len].copy_from_slice(&data[..len]);
        *self = self.slice(len..);
        Ok(len)
    }
}

impl Write for FigBuf<[u8]> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let available = self.len();
        if available == 0 {
            return Err(io::Error::new(
                io::ErrorKind::WriteZero,
                "buffer is full or empty",
            ));
        }

        let to_write = std::cmp::min(buf.len(), available);

        if let Some(slice) = self.try_mut() {
            slice[..to_write].copy_from_slice(&buf[..to_write]);
            Ok(to_write)
        } else {
            Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "buffer is not uniquely owned",
            ))
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_figbuf_from_vec() {
        let vec = vec![1, 2, 3, 4, 5];
        let buf = FigBuf::from_vec(vec);
        assert_eq!(buf.len(), 5);
        assert_eq!(buf.as_slice(), &[1, 2, 3, 4, 5]);
        assert!(!buf.is_static());
    }

    #[test]
    fn test_figbuf_from_static() {
        static DATA: [i32; 5] = [1, 2, 3, 4, 5];
        let buf = FigBuf::<[i32]>::from_static(&DATA);
        assert_eq!(buf.len(), 5);
        assert_eq!(buf.as_slice(), &[1, 2, 3, 4, 5]);
        assert!(buf.is_static());
        assert_eq!(buf.ref_count(), usize::MAX);
    }

    #[test]
    fn test_figbuf_static_clone() {
        static DATA: [i32; 3] = [1, 2, 3];
        let buf = FigBuf::<[i32]>::from_static(&DATA);
        let buf2 = buf.clone();

        assert!(buf.is_static());
        assert!(buf2.is_static());
        assert_eq!(buf.ref_count(), usize::MAX);
        assert_eq!(buf2.ref_count(), usize::MAX);
    }

    #[test]
    fn test_figbuf_static_slice() {
        static DATA: [i32; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let buf = FigBuf::<[i32]>::from_static(&DATA);
        let slice = buf.slice(2..7);

        assert_eq!(slice.as_slice(), &[2, 3, 4, 5, 6]);
        assert!(slice.is_static());
    }

    #[test]
    fn test_figbuf_static_get_mut() {
        static DATA: [i32; 3] = [1, 2, 3];
        let mut buf = FigBuf::<[i32]>::from_static(&DATA);

        // Should return None for static slices
        assert!(buf.get_mut().is_none());
    }

    #[test]
    fn test_figbuf_string_from_static() {
        static TEXT: &str = "Hello, World!";
        let buf = FigBuf::<str>::from_static(TEXT);

        assert_eq!(buf.as_str(), "Hello, World!");
        assert!(buf.is_static());
        assert_eq!(buf.ref_count(), usize::MAX);
    }

    #[test]
    fn test_figbuf_string_static_slice() {
        static TEXT: &str = "Hello, World!";
        let buf = FigBuf::<str>::from_static(TEXT);
        let hello = buf.slice(0..5);
        let world = buf.slice(7..12);

        assert_eq!(hello.as_str(), "Hello");
        assert_eq!(world.as_str(), "World");
        assert!(hello.is_static());
        assert!(world.is_static());
    }

    #[test]
    fn test_figbuf_slice() {
        let buf = FigBuf::from_vec(vec![1, 2, 3, 4, 5]);
        let slice = buf.slice(1..4);
        assert_eq!(slice.len(), 3);
        assert_eq!(slice.as_slice(), &[2, 3, 4]);
    }

    #[test]
    fn test_figbuf_clone() {
        let buf = FigBuf::from_vec(vec![1, 2, 3]);
        let buf2 = buf.clone();
        assert_eq!(buf.ref_count(), 2);
        assert_eq!(buf.as_slice(), buf2.as_slice());
    }

    #[test]
    fn test_figbuf_nested_slice() {
        let buf = FigBuf::from_vec(vec![1, 2, 3, 4, 5, 6, 7, 8]);
        let slice1 = buf.slice(2..7);
        let slice2 = slice1.slice(1..3);
        assert_eq!(slice2.as_slice(), &[4, 5]);
    }

    #[test]
    fn test_figbuf_string() {
        let s = String::from("Hello, World!");
        let buf = FigBuf::from_string(s);
        assert_eq!(buf.as_str(), "Hello, World!");
        assert_eq!(buf.len(), 13);
    }

    #[test]
    fn test_figbuf_string_slice() {
        let buf = FigBuf::from_string(String::from("Hello, World!"));
        let slice = buf.slice(7..12);
        assert_eq!(slice.as_str(), "World");
    }

    #[test]
    fn test_figbuf_deref() {
        let buf = FigBuf::from_vec(vec![1, 2, 3]);
        assert_eq!(&*buf, &[1, 2, 3]);
        assert_eq!(buf[0], 1);
    }

    #[test]
    fn test_figbuf_empty() {
        let buf = FigBuf::from_vec(Vec::<i32>::new());
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);
    }

    #[test]
    fn test_static_empty() {
        static EMPTY: [i32; 0] = [];
        let buf = FigBuf::<[i32]>::from_static(&EMPTY);
        assert!(buf.is_empty());
        assert!(buf.is_static());
    }

    #[test]
    fn test_try_mut_unique_reference() {
        let mut buf = FigBuf::from_vec(vec![1, 2, 3]);
        assert_eq!(buf.ref_count(), 1);

        if let Some(slice) = buf.try_mut() {
            slice[0] = 10;
            slice[1] = 20;
        }

        assert_eq!(&*buf, &[10, 20, 3]);
    }

    #[test]
    fn test_try_mut_shared_reference() {
        let mut buf = FigBuf::from_vec(vec![1, 2, 3]);
        let _clone = buf.clone();

        // Should return None because there are multiple references
        assert!(buf.try_mut().is_none());
    }

    #[test]
    fn test_try_mut_static() {
        static DATA: [i32; 3] = [1, 2, 3];
        let mut buf = FigBuf::<[i32]>::from_static(&DATA);

        // Should return None for static data
        assert!(buf.try_mut().is_none());
    }

    #[test]
    fn test_make_mut_unique() {
        let mut buf = FigBuf::from_vec(vec![1, 2, 3]);
        let initial_ref_count = buf.ref_count();

        let slice = buf.make_mut();
        slice[0] = 10;

        assert_eq!(&*buf, &[10, 2, 3]);
        assert_eq!(buf.ref_count(), initial_ref_count);
    }

    #[test]
    fn test_make_mut_shared() {
        let mut buf = FigBuf::from_vec(vec![1, 2, 3]);
        let clone = buf.clone();

        assert_eq!(buf.ref_count(), 2);

        let slice = buf.make_mut();
        slice[0] = 10;

        assert_eq!(&*buf, &[10, 2, 3]);
        assert_eq!(&*clone, &[1, 2, 3]);
        assert_eq!(buf.ref_count(), 1);
        assert_eq!(clone.ref_count(), 1);
    }

    #[test]
    fn test_make_mut_from_static() {
        static DATA: [i32; 3] = [1, 2, 3];
        let mut buf = FigBuf::<[i32]>::from_static(&DATA);

        assert!(buf.is_static());

        let slice = buf.make_mut();
        slice[0] = 10;

        // Should have cloned from static to heap
        assert_eq!(&*buf, &[10, 2, 3]);
        assert!(!buf.is_static());
        assert_eq!(buf.ref_count(), 1);
    }

    #[test]
    fn test_make_mut_sliced_data() {
        let buf = FigBuf::from_vec(vec![1, 2, 3, 4, 5]);
        let mut slice = buf.slice(1..4);

        assert_eq!(slice.ref_count(), 2);

        let data = slice.make_mut();
        data[0] = 10;

        // slice should have extracted its portion
        assert_eq!(&*slice, &[10, 3, 4]);
        assert_eq!(&*buf, &[1, 2, 3, 4, 5]);
        assert_eq!(slice.ref_count(), 1);
    }

    #[test]
    fn test_string_try_mut_unique() {
        let mut buf = FigBuf::from_string(String::from("hello"));

        if let Some(s) = buf.try_mut() {
            s.make_ascii_uppercase();
        }

        assert_eq!(&*buf, "HELLO");
    }

    #[test]
    fn test_string_try_mut_shared() {
        let mut buf = FigBuf::from_string(String::from("hello"));
        let _clone = buf.clone();

        assert!(buf.try_mut().is_none());
    }

    #[test]
    fn test_string_make_mut_unique() {
        let mut buf = FigBuf::from_string(String::from("hello"));

        let s = buf.make_mut();
        s.make_ascii_uppercase();

        assert_eq!(&*buf, "HELLO");
    }

    #[test]
    fn test_string_make_mut_shared() {
        let mut buf = FigBuf::from_string(String::from("hello"));
        let clone = buf.clone();

        let s = buf.make_mut();
        s.make_ascii_uppercase();

        assert_eq!(&*buf, "HELLO");
        assert_eq!(&*clone, "hello");
    }

    #[test]
    fn test_string_make_mut_from_static() {
        static TEXT: &str = "hello";
        let mut buf = FigBuf::<str>::from_static(TEXT);

        assert!(buf.is_static());

        let s = buf.make_mut();
        s.make_ascii_uppercase();

        assert_eq!(&*buf, "HELLO");
        assert!(!buf.is_static());
    }

    #[test]
    fn test_string_make_mut_sliced() {
        let buf = FigBuf::from_string(String::from("Hello, World!"));
        let mut hello = buf.slice(0..5);

        let s = hello.make_mut();
        s.make_ascii_uppercase();

        assert_eq!(&*hello, "HELLO");
        assert_eq!(&*buf, "Hello, World!");
    }

    #[test]
    fn test_hash_slice() {
        use std::collections::hash_map::DefaultHasher;

        let buf1 = FigBuf::from_vec(vec![1, 2, 3, 4, 5]);
        let buf2 = FigBuf::from_vec(vec![1, 2, 3, 4, 5]);
        let buf3 = FigBuf::from_vec(vec![1, 2, 3, 4, 6]);

        let mut hasher1 = DefaultHasher::new();
        buf1.hash(&mut hasher1);
        let hash1 = hasher1.finish();

        let mut hasher2 = DefaultHasher::new();
        buf2.hash(&mut hasher2);
        let hash2 = hasher2.finish();

        let mut hasher3 = DefaultHasher::new();
        buf3.hash(&mut hasher3);
        let hash3 = hasher3.finish();

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_hash_slice_with_slicing() {
        use std::collections::hash_map::DefaultHasher;

        let buf = FigBuf::from_vec(vec![1, 2, 3, 4, 5]);
        let slice = buf.slice(1..4);

        let direct = FigBuf::from_vec(vec![2, 3, 4]);

        let mut hasher1 = DefaultHasher::new();
        slice.hash(&mut hasher1);
        let hash1 = hasher1.finish();

        let mut hasher2 = DefaultHasher::new();
        direct.hash(&mut hasher2);
        let hash2 = hasher2.finish();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_string() {
        use std::collections::hash_map::DefaultHasher;

        let buf1 = FigBuf::from_string(String::from("hello"));
        let buf2 = FigBuf::from_string(String::from("hello"));
        let buf3 = FigBuf::from_string(String::from("world"));

        let mut hasher1 = DefaultHasher::new();
        buf1.hash(&mut hasher1);
        let hash1 = hasher1.finish();

        let mut hasher2 = DefaultHasher::new();
        buf2.hash(&mut hasher2);
        let hash2 = hasher2.finish();

        let mut hasher3 = DefaultHasher::new();
        buf3.hash(&mut hasher3);
        let hash3 = hasher3.finish();

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_hash_string_with_slicing() {
        use std::collections::hash_map::DefaultHasher;

        let buf = FigBuf::from_string(String::from("Hello, World!"));
        let slice = buf.slice(7..12);

        let direct = FigBuf::from_string(String::from("World"));

        let mut hasher1 = DefaultHasher::new();
        slice.hash(&mut hasher1);
        let hash1 = hasher1.finish();

        let mut hasher2 = DefaultHasher::new();
        direct.hash(&mut hasher2);
        let hash2 = hasher2.finish();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_static() {
        use std::collections::hash_map::DefaultHasher;

        static DATA: [i32; 5] = [1, 2, 3, 4, 5];
        let buf1 = FigBuf::<[i32]>::from_static(&DATA);
        let buf2 = FigBuf::from_vec(vec![1, 2, 3, 4, 5]);

        let mut hasher1 = DefaultHasher::new();
        buf1.hash(&mut hasher1);
        let hash1 = hasher1.finish();

        let mut hasher2 = DefaultHasher::new();
        buf2.hash(&mut hasher2);
        let hash2 = hasher2.finish();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hashmap_usage() {
        use std::collections::HashMap;

        let mut map = HashMap::new();
        let key1 = FigBuf::from_vec(vec![1, 2, 3]);
        let key2 = FigBuf::from_vec(vec![1, 2, 3]);
        let key3 = FigBuf::from_vec(vec![4, 5, 6]);

        map.insert(key1.clone(), "first");
        map.insert(key3.clone(), "second");

        assert_eq!(map.get(&key2), Some(&"first"));
        assert_eq!(map.get(&key1), Some(&"first"));
        assert_eq!(map.get(&key3), Some(&"second"));
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_hashmap_string_usage() {
        use std::collections::HashMap;

        let mut map = HashMap::new();
        let key1 = FigBuf::from_string(String::from("hello"));
        let key2 = FigBuf::from_string(String::from("hello"));
        let key3 = FigBuf::from_string(String::from("world"));

        map.insert(key1.clone(), 42);
        map.insert(key3.clone(), 99);

        assert_eq!(map.get(&key2), Some(&42));
        assert_eq!(map.get(&key1), Some(&42));
        assert_eq!(map.get(&key3), Some(&99));
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_equality_slice() {
        let buf1 = FigBuf::from_vec(vec![1, 2, 3]);
        let buf2 = FigBuf::from_vec(vec![1, 2, 3]);
        let buf3 = FigBuf::from_vec(vec![1, 2, 4]);

        assert_eq!(buf1, buf2);
        assert_ne!(buf1, buf3);
    }

    #[test]
    fn test_equality_string() {
        let buf1 = FigBuf::from_string(String::from("hello"));
        let buf2 = FigBuf::from_string(String::from("hello"));
        let buf3 = FigBuf::from_string(String::from("world"));

        assert_eq!(buf1, buf2);
        assert_ne!(buf1, buf3);
    }

    #[test]
    fn test_equality_with_slicing() {
        let buf = FigBuf::from_vec(vec![1, 2, 3, 4, 5]);
        let slice = buf.slice(1..4);
        let direct = FigBuf::from_vec(vec![2, 3, 4]);

        assert_eq!(slice, direct);
    }

    #[test]
    fn test_borrow_slice() {
        use std::collections::HashMap;

        let mut map: HashMap<Vec<i32>, &str> = HashMap::new();
        map.insert(vec![1, 2, 3], "test");

        let buf = FigBuf::from_vec(vec![1, 2, 3]);
        let borrowed: &[i32] = buf.borrow();
        assert_eq!(map.get(borrowed), Some(&"test"));
    }

    #[test]
    fn test_borrow_str() {
        use std::collections::HashMap;

        let mut map: HashMap<String, i32> = HashMap::new();
        map.insert(String::from("hello"), 42);

        let buf = FigBuf::from_string(String::from("hello"));
        let borrowed: &str = buf.borrow();
        assert_eq!(map.get(borrowed), Some(&42));
    }

    #[test]
    fn test_read_trait() {
        use std::io::Read;

        let mut buf = FigBuf::from_vec(vec![1, 2, 3, 4, 5]);
        let mut output = [0u8; 3];

        let bytes_read = buf.read(&mut output).unwrap();
        assert_eq!(bytes_read, 3);
        assert_eq!(output, [1, 2, 3]);
        assert_eq!(buf.as_slice(), &[4, 5]);
    }

    #[test]
    fn test_read_trait_partial() {
        use std::io::Read;

        let mut buf = FigBuf::from_vec(vec![1, 2, 3]);
        let mut output = [0u8; 5];

        let bytes_read = buf.read(&mut output).unwrap();
        assert_eq!(bytes_read, 3);
        assert_eq!(&output[..3], &[1, 2, 3]);
        assert_eq!(buf.len(), 0);
    }

    #[test]
    fn test_read_trait_multiple_reads() {
        use std::io::Read;

        let mut buf = FigBuf::from_vec(vec![1, 2, 3, 4, 5, 6, 7, 8]);
        let mut output1 = [0u8; 3];
        let mut output2 = [0u8; 3];
        let mut output3 = [0u8; 3];

        assert_eq!(buf.read(&mut output1).unwrap(), 3);
        assert_eq!(output1, [1, 2, 3]);

        assert_eq!(buf.read(&mut output2).unwrap(), 3);
        assert_eq!(output2, [4, 5, 6]);

        assert_eq!(buf.read(&mut output3).unwrap(), 2);
        assert_eq!(&output3[..2], &[7, 8]);
    }

    #[test]
    fn test_write_trait() {
        use std::io::Write;

        let mut buf = FigBuf::from_vec(vec![0u8; 5]);
        let data = [1, 2, 3];

        let bytes_written = buf.write(&data).unwrap();
        assert_eq!(bytes_written, 3);
        assert_eq!(buf.as_slice(), &[1, 2, 3, 0, 0]);
    }

    #[test]
    fn test_write_trait_partial() {
        use std::io::Write;

        let mut buf = FigBuf::from_vec(vec![0u8; 3]);
        let data = [1, 2, 3, 4, 5];

        let bytes_written = buf.write(&data).unwrap();
        assert_eq!(bytes_written, 3);
        assert_eq!(buf.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn test_write_trait_shared_fails() {
        use std::io::Write;

        let mut buf = FigBuf::from_vec(vec![0u8; 5]);
        let _clone = buf.clone();
        let data = [1, 2, 3];

        let result = buf.write(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_write_trait_empty_fails() {
        use std::io::Write;

        let mut buf = FigBuf::from_vec(vec![]);
        let data = [1, 2, 3];

        let result = buf.write(&data);
        assert!(result.is_err());
    }
}
