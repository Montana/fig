use crate::FigBuf;
use std::fmt;
use std::ops::{Deref, RangeBounds};

#[derive(Clone)]
pub struct Bytes {
    inner: FigBuf<[u8]>,
}

impl Bytes {

    pub fn from_vec(vec: Vec<u8>) -> Self {
        Self {
            inner: FigBuf::from_vec(vec),
        }
    }

    pub fn new() -> Self {
        Self {
            inner: FigBuf::from_vec(Vec::new()),
        }
    }

    pub fn from_static(bytes: &'static [u8]) -> Self {
        Self {
            inner: FigBuf::from_vec(bytes.to_vec()),
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn slice(&self, range: impl RangeBounds<usize>) -> Self {
        Self {
            inner: self.inner.slice(range),
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        self.inner.as_slice()
    }

    pub fn try_into_vec(mut self) -> Result<Vec<u8>, Self> {
        match self.inner.get_mut() {
            Some(slice) => Ok(slice.to_vec()),
            None => Err(self),
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.inner.as_slice().to_vec()
    }

    pub fn split_off(&mut self, at: usize) -> Self {
        let right = self.inner.slice(at..);
        self.inner = self.inner.slice(..at);
        Self { inner: right }
    }

    
    pub fn split_to(&mut self, at: usize) -> Self {
        let left = self.inner.slice(..at);
        self.inner = self.inner.slice(at..);
        Self { inner: left }
    }

    pub fn truncate(&mut self, len: usize) {
        if len < self.len() {
            self.inner = self.inner.slice(..len);
        }
    }

    pub fn clear(&mut self) {
        self.inner = FigBuf::from_vec(Vec::new());
    }
}

impl Default for Bytes {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for Bytes {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.inner.as_slice()
    }
}

impl AsRef<[u8]> for Bytes {
    fn as_ref(&self) -> &[u8] {
        self.inner.as_slice()
    }
}

impl From<Vec<u8>> for Bytes {
    fn from(vec: Vec<u8>) -> Self {
        Self::from_vec(vec)
    }
}

impl From<&'static [u8]> for Bytes {
    fn from(slice: &'static [u8]) -> Self {
        Self::from_static(slice)
    }
}

impl From<&'static str> for Bytes {
    fn from(s: &'static str) -> Self {
        Self::from_static(s.as_bytes())
    }
}

impl From<String> for Bytes {
    fn from(s: String) -> Self {
        Self::from_vec(s.into_bytes())
    }
}

impl fmt::Debug for Bytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_slice(), f)
    }
}

impl PartialEq for Bytes {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl Eq for Bytes {}

impl PartialEq<[u8]> for Bytes {
    fn eq(&self, other: &[u8]) -> bool {
        self.as_slice() == other
    }
}

impl PartialEq<Bytes> for [u8] {
    fn eq(&self, other: &Bytes) -> bool {
        self == other.as_slice()
    }
}

impl PartialEq<Vec<u8>> for Bytes {
    fn eq(&self, other: &Vec<u8>) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl PartialEq<Bytes> for Vec<u8> {
    fn eq(&self, other: &Bytes) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl PartialEq<&[u8]> for Bytes {
    fn eq(&self, other: &&[u8]) -> bool {
        self.as_slice() == *other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytes_creation() {
        let bytes = Bytes::from_vec(vec![1, 2, 3, 4, 5]);
        assert_eq!(bytes.len(), 5);
        assert_eq!(&*bytes, &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_bytes_slice() {
        let bytes = Bytes::from_vec(vec![1, 2, 3, 4, 5]);
        let slice = bytes.slice(1..4);
        assert_eq!(&*slice, &[2, 3, 4]);
    }

    #[test]
    fn test_bytes_split_off() {
        let mut bytes = Bytes::from_vec(vec![1, 2, 3, 4, 5]);
        let right = bytes.split_off(3);
        assert_eq!(&*bytes, &[1, 2, 3]);
        assert_eq!(&*right, &[4, 5]);
    }

    #[test]
    fn test_bytes_split_to() {
        let mut bytes = Bytes::from_vec(vec![1, 2, 3, 4, 5]);
        let left = bytes.split_to(3);
        assert_eq!(&*left, &[1, 2, 3]);
        assert_eq!(&*bytes, &[4, 5]);
    }

    #[test]
    fn test_bytes_truncate() {
        let mut bytes = Bytes::from_vec(vec![1, 2, 3, 4, 5]);
        bytes.truncate(3);
        assert_eq!(&*bytes, &[1, 2, 3]);
    }

    #[test]
    fn test_bytes_equality() {
        let bytes1 = Bytes::from_vec(vec![1, 2, 3]);
        let bytes2 = Bytes::from_vec(vec![1, 2, 3]);
        let bytes3 = Bytes::from_vec(vec![1, 2, 4]);

        assert_eq!(bytes1, bytes2);
        assert_ne!(bytes1, bytes3);
        assert_eq!(bytes1, vec![1, 2, 3]);
        assert_eq!(bytes1, &[1, 2, 3][..]);
    }

    #[test]
    fn test_bytes_from_string() {
        let bytes = Bytes::from(String::from("hello"));
        assert_eq!(&*bytes, b"hello");
    }

    #[test]
    fn test_bytes_empty() {
        let bytes = Bytes::new();
        assert!(bytes.is_empty());
        assert_eq!(bytes.len(), 0);
    }
}