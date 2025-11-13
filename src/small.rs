//! Small buffer optimization for inline storage of small data.
//!
//! `SmallFigBuf` can store small byte slices inline without heap allocation,
//! falling back to heap storage for larger data.

use crate::FigBuf;
use std::convert::Infallible;
use std::fmt;
use std::ops::{Deref, RangeBounds};
use std::str::FromStr;

/// Internal representation of small buffer data.
enum SmallInner<const N: usize> {
    /// Data stored inline within the struct (no heap allocation).
    Inline {
        data: [u8; N],
        len: usize,
    },
    /// Data stored on the heap via FigBuf.
    Heap(FigBuf<[u8]>),
}

/// A byte buffer with small buffer optimization.
///
/// `SmallFigBuf<N>` can store up to `N` bytes inline without heap allocation.
/// When data exceeds `N` bytes, it falls back to heap storage using `FigBuf`.
///
/// This is particularly useful for:
/// - Short strings and identifiers
/// - Small message payloads
/// - Cache-friendly data structures
///
/// # Example
///
/// ```
/// use fig::small::SmallFigBuf;
///
/// // Small data stored inline (no heap allocation)
/// let small: SmallFigBuf<32> = SmallFigBuf::from_slice(b"hello");
/// assert!(small.is_inline());
///
/// // Large data uses heap storage
/// let large: SmallFigBuf<32> = SmallFigBuf::from_slice(&[0; 100]);
/// assert!(!large.is_inline());
/// ```
pub struct SmallFigBuf<const N: usize> {
    inner: SmallInner<N>,
}

impl<const N: usize> SmallFigBuf<N> {
    /// Creates a new empty `SmallFigBuf`.
    pub fn new() -> Self {
        Self {
            inner: SmallInner::Inline {
                data: [0; N],
                len: 0,
            },
        }
    }

    /// Creates a `SmallFigBuf` from a byte slice.
    ///
    /// If the slice fits within `N` bytes, it's stored inline.
    /// Otherwise, it's allocated on the heap.
    pub fn from_slice(slice: &[u8]) -> Self {
        if slice.len() <= N {
            let mut data = [0; N];
            data[..slice.len()].copy_from_slice(slice);
            Self {
                inner: SmallInner::Inline {
                    data,
                    len: slice.len(),
                },
            }
        } else {
            Self {
                inner: SmallInner::Heap(FigBuf::from_vec(slice.to_vec())),
            }
        }
    }

    /// Creates a `SmallFigBuf` from a static slice without allocation.
    pub fn from_static(slice: &'static [u8]) -> Self {
        Self {
            inner: SmallInner::Heap(FigBuf::<[u8]>::from_static(slice)),
        }
    }

    /// Creates a `SmallFigBuf` from a vector.
    ///
    /// If the vector fits within `N` bytes, data is copied inline.
    /// Otherwise, the vector is moved to heap storage.
    pub fn from_vec(vec: Vec<u8>) -> Self {
        if vec.len() <= N {
            Self::from_slice(&vec)
        } else {
            Self {
                inner: SmallInner::Heap(FigBuf::from_vec(vec)),
            }
        }
    }

    /// Returns the number of bytes in the buffer.
    pub fn len(&self) -> usize {
        match &self.inner {
            SmallInner::Inline { len, .. } => *len,
            SmallInner::Heap(buf) => buf.len(),
        }
    }

    /// Returns `true` if the buffer has a length of 0.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns `true` if the data is stored inline (not on the heap).
    pub fn is_inline(&self) -> bool {
        matches!(&self.inner, SmallInner::Inline { .. })
    }

    /// Returns `true` if the data is stored on the heap.
    pub fn is_heap(&self) -> bool {
        matches!(&self.inner, SmallInner::Heap(_))
    }

    /// Returns a reference to the underlying byte slice.
    pub fn as_slice(&self) -> &[u8] {
        match &self.inner {
            SmallInner::Inline { data, len } => &data[..*len],
            SmallInner::Heap(buf) => buf.as_slice(),
        }
    }

    /// Creates a new `SmallFigBuf` representing a subslice.
    ///
    /// If currently inline, the slice is created inline if it still fits.
    /// Otherwise, uses `FigBuf`'s zero-copy slicing.
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
            Bound::Unbounded => self.len(),
        };

        assert!(start <= end, "slice start must be <= end");
        assert!(end <= self.len(), "slice end out of bounds");

        let slice_len = end - start;

        match &self.inner {
            SmallInner::Inline { data, len: _ } => {
                if slice_len <= N {
                    // Keep inline
                    let mut new_data = [0; N];
                    new_data[..slice_len].copy_from_slice(&data[start..end]);
                    Self {
                        inner: SmallInner::Inline {
                            data: new_data,
                            len: slice_len,
                        },
                    }
                } else {
                    unreachable!("slice of inline data cannot exceed capacity")
                }
            }
            SmallInner::Heap(buf) => {
                // Use FigBuf's zero-copy slicing
                Self {
                    inner: SmallInner::Heap(buf.slice(start..end)),
                }
            }
        }
    }

    /// Converts to a `FigBuf<[u8]>`.
    ///
    /// If inline, allocates and copies data to the heap.
    /// If already heap, returns a clone of the underlying `FigBuf`.
    pub fn to_figbuf(&self) -> FigBuf<[u8]> {
        match &self.inner {
            SmallInner::Inline { data, len } => FigBuf::from_vec(data[..*len].to_vec()),
            SmallInner::Heap(buf) => buf.clone(),
        }
    }

    /// Returns the capacity of inline storage.
    pub const fn inline_capacity() -> usize {
        N
    }

    /// Spills inline data to the heap, returning a `FigBuf`.
    ///
    /// If already on heap, returns a clone.
    pub fn into_figbuf(self) -> FigBuf<[u8]> {
        match self.inner {
            SmallInner::Inline { data, len } => FigBuf::from_vec(data[..len].to_vec()),
            SmallInner::Heap(buf) => buf,
        }
    }
}

impl<const N: usize> Clone for SmallFigBuf<N> {
    fn clone(&self) -> Self {
        match &self.inner {
            SmallInner::Inline { data, len } => Self {
                inner: SmallInner::Inline {
                    data: *data,
                    len: *len,
                },
            },
            SmallInner::Heap(buf) => Self {
                inner: SmallInner::Heap(buf.clone()),
            },
        }
    }
}

impl<const N: usize> Default for SmallFigBuf<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Deref for SmallFigBuf<N> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<const N: usize> AsRef<[u8]> for SmallFigBuf<N> {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl<const N: usize> fmt::Debug for SmallFigBuf<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SmallFigBuf")
            .field("len", &self.len())
            .field("inline", &self.is_inline())
            .field("data", &self.as_slice())
            .finish()
    }
}

impl<const N: usize> PartialEq for SmallFigBuf<N> {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<const N: usize> Eq for SmallFigBuf<N> {}

impl<const N: usize> PartialEq<[u8]> for SmallFigBuf<N> {
    fn eq(&self, other: &[u8]) -> bool {
        self.as_slice() == other
    }
}

impl<const N: usize> PartialEq<&[u8]> for SmallFigBuf<N> {
    fn eq(&self, other: &&[u8]) -> bool {
        self.as_slice() == *other
    }
}

impl<const N: usize, const M: usize> PartialEq<&[u8; M]> for SmallFigBuf<N> {
    fn eq(&self, other: &&[u8; M]) -> bool {
        self.as_slice() == &other[..]
    }
}

impl<const N: usize> From<Vec<u8>> for SmallFigBuf<N> {
    fn from(vec: Vec<u8>) -> Self {
        Self::from_vec(vec)
    }
}

impl<const N: usize> From<&[u8]> for SmallFigBuf<N> {
    fn from(slice: &[u8]) -> Self {
        Self::from_slice(slice)
    }
}

impl<const N: usize> From<&str> for SmallFigBuf<N> {
    fn from(s: &str) -> Self {
        Self::from_slice(s.as_bytes())
    }
}

/// A string with small buffer optimization.
///
/// `SmallFigStr<N>` can store up to `N` bytes of UTF-8 data inline without heap allocation.
pub struct SmallFigStr<const N: usize> {
    inner: SmallFigBuf<N>,
}

impl<const N: usize> SmallFigStr<N> {
    /// Creates a new empty `SmallFigStr`.
    pub fn new() -> Self {
        Self {
            inner: SmallFigBuf::new(),
        }
    }

    /// Creates a `SmallFigStr` from a static string without allocation.
    pub fn from_static(s: &'static str) -> Self {
        Self {
            inner: SmallFigBuf::from_static(s.as_bytes()),
        }
    }

    /// Returns the length of the string in bytes.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` if the string has a length of 0.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns `true` if the data is stored inline (not on the heap).
    pub fn is_inline(&self) -> bool {
        self.inner.is_inline()
    }

    /// Returns a reference to the underlying string slice.
    pub fn as_str(&self) -> &str {
        // SAFETY: SmallFigStr only accepts valid UTF-8
        unsafe { std::str::from_utf8_unchecked(self.inner.as_slice()) }
    }

    /// Creates a new `SmallFigStr` representing a substring.
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
            Bound::Unbounded => self.len(),
        };

        assert!(self.as_str().is_char_boundary(start), "slice start not at char boundary");
        assert!(self.as_str().is_char_boundary(end), "slice end not at char boundary");

        Self {
            inner: self.inner.slice(start..end),
        }
    }
}

impl<const N: usize> Clone for SmallFigStr<N> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<const N: usize> Default for SmallFigStr<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Deref for SmallFigStr<N> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl<const N: usize> AsRef<str> for SmallFigStr<N> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<const N: usize> fmt::Display for SmallFigStr<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

impl<const N: usize> fmt::Debug for SmallFigStr<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SmallFigStr")
            .field("len", &self.len())
            .field("inline", &self.is_inline())
            .field("data", &self.as_str())
            .finish()
    }
}

impl<const N: usize> PartialEq for SmallFigStr<N> {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl<const N: usize> Eq for SmallFigStr<N> {}

impl<const N: usize> PartialEq<str> for SmallFigStr<N> {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl<const N: usize> PartialEq<&str> for SmallFigStr<N> {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl<const N: usize> From<&str> for SmallFigStr<N> {
    fn from(s: &str) -> Self {
        Self {
            inner: SmallFigBuf::from_slice(s.as_bytes()),
        }
    }
}

impl<const N: usize> From<String> for SmallFigStr<N> {
    fn from(s: String) -> Self {
        Self {
            inner: SmallFigBuf::from_slice(s.as_bytes()),
        }
    }
}

impl<const N: usize> FromStr for SmallFigStr<N> {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            inner: SmallFigBuf::from_slice(s.as_bytes()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_inline() {
        let buf: SmallFigBuf<32> = SmallFigBuf::from_slice(b"hello");
        assert!(buf.is_inline());
        assert_eq!(buf.len(), 5);
        assert_eq!(&*buf, b"hello");
    }

    #[test]
    fn test_small_heap() {
        let large_data = vec![0u8; 100];
        let buf: SmallFigBuf<32> = SmallFigBuf::from_vec(large_data);
        assert!(!buf.is_inline());
        assert!(buf.is_heap());
        assert_eq!(buf.len(), 100);
    }

    #[test]
    fn test_small_exact_capacity() {
        let buf: SmallFigBuf<5> = SmallFigBuf::from_slice(b"hello");
        assert!(buf.is_inline());
        assert_eq!(buf.len(), 5);
    }

    #[test]
    fn test_small_over_capacity() {
        let buf: SmallFigBuf<4> = SmallFigBuf::from_slice(b"hello");
        assert!(!buf.is_inline());
        assert_eq!(buf.len(), 5);
    }

    #[test]
    fn test_small_clone() {
        let buf1: SmallFigBuf<32> = SmallFigBuf::from_slice(b"test");
        let buf2 = buf1.clone();

        assert_eq!(buf1, buf2);
        assert!(buf1.is_inline());
        assert!(buf2.is_inline());
    }

    #[test]
    fn test_small_slice() {
        let buf: SmallFigBuf<32> = SmallFigBuf::from_slice(b"hello world");
        let slice = buf.slice(0..5);

        assert_eq!(&*slice, b"hello");
        assert!(slice.is_inline());
    }

    #[test]
    fn test_small_empty() {
        let buf: SmallFigBuf<32> = SmallFigBuf::new();
        assert!(buf.is_empty());
        assert!(buf.is_inline());
        assert_eq!(buf.len(), 0);
    }

    #[test]
    fn test_small_str_inline() {
        let s: SmallFigStr<32> = SmallFigStr::from("hello");
        assert!(s.is_inline());
        assert_eq!(&*s, "hello");
    }

    #[test]
    fn test_small_str_heap() {
        let long = "a".repeat(100);
        let s: SmallFigStr<32> = SmallFigStr::from(&long[..]);
        assert!(!s.is_inline());
        assert_eq!(s.len(), 100);
    }

    #[test]
    fn test_small_str_slice() {
        let s: SmallFigStr<32> = SmallFigStr::from("hello world");
        let slice = s.slice(0..5);
        assert_eq!(&*slice, "hello");
    }

    #[test]
    fn test_small_str_static() {
        static TEXT: &str = "static text";
        let s: SmallFigStr<32> = SmallFigStr::from_static(TEXT);
        assert_eq!(&*s, "static text");
    }
}
