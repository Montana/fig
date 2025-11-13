use std::fmt;
use std::ops::{Deref, RangeBounds};
use std::sync::Arc;

pub mod bytes;
pub mod small;

/// Internal representation of the data backing a `FigBuf`.
enum Inner<T: ?Sized + 'static> {
    /// Static data that doesn't require reference counting.
    Static(&'static T),
    /// Reference-counted data on the heap.
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

/// A reference-counted shared slice of generic data.
///
/// `FigBuf<T>` provides a way to share slices of data between multiple owners
/// with efficient cloning and slicing operations. Similar to `Arc<[T]>` but
/// with additional slicing capabilities that maintain shared ownership.
///
/// Static slices are stored without heap allocation, providing zero-cost
/// abstraction for compile-time known data.
pub struct FigBuf<T: ?Sized + 'static> {
    inner: Inner<T>,
    offset: usize,
    len: usize,
}

impl<T: 'static> FigBuf<[T]> {
    /// Creates a new `FigBuf` from a vector.
    pub fn from_vec(vec: Vec<T>) -> Self {
        let len = vec.len();
        Self {
            inner: Inner::Arc(Arc::from(vec.into_boxed_slice())),
            offset: 0,
            len,
        }
    }

    /// Creates a new `FigBuf` from a boxed slice.
    pub fn from_boxed_slice(slice: Box<[T]>) -> Self {
        let len = slice.len();
        Self {
            inner: Inner::Arc(Arc::from(slice)),
            offset: 0,
            len,
        }
    }

    /// Creates a new `FigBuf` from a static slice without heap allocation.
    ///
    /// This is a zero-cost operation as it doesn't allocate an Arc.
    pub fn from_static(slice: &'static [T]) -> Self {
        Self {
            inner: Inner::Static(slice),
            offset: 0,
            len: slice.len(),
        }
    }

    /// Returns the number of elements in the slice.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the slice has a length of 0.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Creates a new `FigBuf` that shares the same underlying data but
    /// represents a subslice of the original.
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

    /// Returns a reference to the underlying slice.
    pub fn as_slice(&self) -> &[T] {
        let full_slice = match &self.inner {
            Inner::Static(s) => s,
            Inner::Arc(arc) => &**arc,
        };
        &full_slice[self.offset..self.offset + self.len]
    }

    /// Attempts to get a mutable reference to the underlying data.
    /// Returns `Some(&mut [T])` if this is the only reference to the data.
    ///
    /// Note: This always returns `None` for static slices.
    pub fn get_mut(&mut self) -> Option<&mut [T]> {
        match &mut self.inner {
            Inner::Static(_) => None,
            Inner::Arc(arc) => {
                Arc::get_mut(arc).map(|slice| &mut slice[self.offset..self.offset + self.len])
            }
        }
    }

    /// Returns the number of references to the underlying data.
    ///
    /// For static slices, this always returns `usize::MAX` to indicate
    /// the data is effectively immortal.
    pub fn ref_count(&self) -> usize {
        match &self.inner {
            Inner::Static(_) => usize::MAX,
            Inner::Arc(arc) => Arc::strong_count(arc),
        }
    }

    /// Returns `true` if this `FigBuf` is backed by a static slice.
    pub fn is_static(&self) -> bool {
        matches!(&self.inner, Inner::Static(_))
    }
}

impl FigBuf<str> {
    /// Creates a new `FigBuf` from a `String`.
    pub fn from_string(s: String) -> Self {
        let bytes = FigBuf::from_vec(s.into_bytes());
        Self {
            inner: match bytes.inner {
                Inner::Arc(arc) => Inner::Arc(unsafe {
                    // SAFETY: We know the bytes came from a valid String
                    Arc::from_raw(Arc::into_raw(arc) as *const str)
                }),
                Inner::Static(_) => unreachable!("from_vec never returns Static"),
            },
            offset: 0,
            len: bytes.len,
        }
    }

    /// Creates a new `FigBuf` from a static string without heap allocation.
    ///
    /// This is a zero-cost operation as it doesn't allocate an Arc.
    pub fn from_static(s: &'static str) -> Self {
        Self {
            inner: Inner::Static(s),
            offset: 0,
            len: s.len(),
        }
    }

    /// Returns the length of the string in bytes.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the string has a length of 0.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Creates a new `FigBuf` that shares the same underlying data but
    /// represents a substring of the original.
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

    /// Returns a reference to the underlying string slice.
    pub fn as_str(&self) -> &str {
        let full_str = match &self.inner {
            Inner::Static(s) => s,
            Inner::Arc(arc) => &**arc,
        };
        &full_str[self.offset..self.offset + self.len]
    }

    /// Returns the number of references to the underlying data.
    ///
    /// For static strings, this always returns `usize::MAX` to indicate
    /// the data is effectively immortal.
    pub fn ref_count(&self) -> usize {
        match &self.inner {
            Inner::Static(_) => usize::MAX,
            Inner::Arc(arc) => Arc::strong_count(arc),
        }
    }

    /// Returns `true` if this `FigBuf` is backed by a static string.
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
}
