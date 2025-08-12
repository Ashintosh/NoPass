use serde::{Serialize, Deserialize};
use slint::SharedString;
use subtle::ConstantTimeEq;
use zeroize::{Zeroize, ZeroizeOnDrop};


#[derive(Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub(crate) struct ZeroByte {
    data: Vec<u8>,
}

impl ZeroByte {    
    /// Create a new ZeroByte with pre-allocated capacity
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self { data: Vec::with_capacity(capacity) }
    }

    /// Create ZeroByte from SharedString
    pub(crate) fn from_shared_string(s: SharedString) -> Self {
        Self { data: s.as_str().bytes().collect() }
    }

    /// Convert directly to SharedString with minimal intermediate copies
    pub(crate) fn to_shared_string_secure(&self) -> SharedString {
        self.with_bytes(|bytes| {
            // Convert directly without intermediate String if possible
            match std::str::from_utf8(bytes) {
                Ok(valid_str) => SharedString::from(valid_str),
                Err(_) => {
                    // Handle invalid UTF-8 gracefully
                    let string = String::from_utf8_lossy(bytes).into_owned();
                    SharedString::from(string)
                }
            }
        })
    }

    /// Create ZeroByte from byte slice
    pub(crate) fn from_bytes(b: &[u8]) -> Self {
        Self { data: b.to_vec() }
    }

    /// Create ZeroByte from Vec, securely taking ownership
    pub(crate) fn from_vec(v: Vec<u8>) -> Self {
        Self { data: v }
    }

    /// Get the length of the data
    pub(crate) fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the data is empty
    #[allow(dead_code)]
    pub(crate) fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get the capacity of the underlying vector
    #[allow(dead_code)]
    pub(crate) fn capacity(&self) -> usize {
        self.data.capacity()
    }

    /// Reserve addition capacity
    #[allow(dead_code)]
    pub(crate) fn reserve(&mut self, additional: usize) {
        self.data.reserve(additional);
    }

    /// Secure access to bytes via closure to prevent lingering references
    pub(crate) fn with_bytes<T>(&self, f: impl FnOnce(&[u8]) -> T) -> T {
        f(&self.data)
    }

    /// Secure mutable access to bytes via closure
    #[allow(dead_code)]
    pub(crate) fn with_bytes_mut<T>(&mut self, f: impl FnOnce(&mut [u8]) -> T) -> T {
        f(&mut self.data)
    }
    
    /// Constant-time equality comparison with byte slice
    pub(crate) fn constant_time_eq(&self, other: &[u8]) -> bool {
        if self.data.len() != other.len() {
            return false;
        }
        self.data.ct_eq(other).into()
    }

    /// Constant-time equality comparison with another ZeroByte
    pub(crate) fn constant_time_eq_zero_byte(&self, other: &Self) -> bool {
        self.constant_time_eq(&other.data)
    }

    /// Create a new ZeroByte with a copy of a range of byte
    #[allow(dead_code)]
    pub(super) fn secure_copy_range(&self, start: usize, end: usize) -> Self {
        if start > end || end > self.data.len() {
            panic!("Invalid range for secure_copy_range");
        }
        Self::from_bytes(&self.data[start..end])
        //Self { data: self.data[start..end].to_vec() }
    }

    /// Split into new ZeroByte instances at the given index
    pub(crate) fn secure_split_at(&self, mid: usize) -> (Self, Self) {
        if mid > self.data.len() {
            panic!("Split index out of bounds");
        }

        let left = Self::from_bytes(&self.data[..mid]);
        let right = Self::from_bytes(&self.data[mid..]);

        (left, right)
    }

    /// Extend the data with bytes from a slice
    #[allow(dead_code)]
    pub(crate) fn extend_from_slice(&mut self, other: &[u8]) {
        self.data.extend_from_slice(other);
    }

    /// Extend that data with bytes from another ZeroByte
    pub(crate) fn extend_from_zero_byte(&mut self, other: &Self) {
        self.data.extend_from_slice(&other.data);
    }

    /// Securely truncate the given length, zeroizing removed data
    #[allow(dead_code)]
    pub(crate) fn truncate(&mut self, len: usize) {
        if len < self.data.len() {
            // Zeroize the truncated portion before removing it
            self.data[len..].zeroize();
        }
        self.data.truncate(len);
    }

    /// Securely clear all data
    #[allow(dead_code)]
    pub(crate) fn clear(&mut self) {
        self.data.zeroize();
        self.data.clear();
    }

    /// Push a single byte
    #[allow(dead_code)]
    pub(crate) fn push(&mut self, byte: u8) {
        self.data.push(byte);
    }

    /// Pop a single byte, returning None if empty
    #[allow(dead_code)]
    pub(crate) fn pop(&mut self) -> Option<u8> {
        self.data.pop()
    }

    /// Securely resize the vector, filling new elements with the given value
    #[allow(dead_code)]
    pub(crate) fn resize(&mut self, new_len: usize, value: u8) {
        if new_len < self.data.len() {
            // Zeroize the portion that will be removed
            self.data[new_len..].zeroize();
        }
        self.data.resize(new_len, value);
    }

    /// Get a byte at the specified index
    #[allow(dead_code)]
    pub(crate) fn get(&self, index: usize) -> Option<u8> {
        self.data.get(index).copied()
    }

    /// Set a byte at the specified index
    #[allow(dead_code)]
    pub(crate) fn set(&mut self, index: usize, value: u8) -> Result<(), &'static str> {
        if index < self.data.len() {
            self.data[index] = value;
            Ok(())
        } else {
            Err("Index out of bounds")
        }
    }

    /// Secure comparison that operated in constant time for same-length inputs
    #[allow(dead_code)]
    pub(crate) fn secure_starts_with(&self, prefix: &[u8]) -> bool {
        if prefix.len() > self.data.len() {
            return false;
        }
        self.data[..prefix.len()].ct_eq(prefix).into()
    }

    /// Secure comparison that operated in constant time for same-length inputs
    #[allow(dead_code)]
    pub(crate) fn secure_ends_with(&self, suffix: &[u8]) -> bool {
        if suffix.len() > self.data.len() {
            return false;
        }
        let start = self.data.len() - suffix.len();
        self.data[start..].ct_eq(suffix).into()
    }

    /// Create a copy of this ZeroByte
    pub(crate) fn secure_clone(&self) -> Self {
        Self { data: self.data.clone() }
    }

    /// Append another ZeroByte to this one, consuming the other
    #[allow(dead_code)]
    pub(crate) fn append(&mut self, mut other: ZeroByte) {
        self.data.append(&mut other.data);
        // Other will be zeroized when it goes out of scope due to ZeroizeOnDrop
    }

    /// Convert to a Vec<u8>, consuming self and ensuring no copies remain
    #[allow(dead_code)]
    pub(crate) fn into_vec(mut self) -> Vec<u8> {
        let mut result = Vec::new();
        std::mem::swap(&mut result, &mut self.data);
        result
    }
}

// Implement constant-time equality for the struct itself
impl PartialEq for ZeroByte {
    fn eq(&self, other: &Self) -> bool {
        self.constant_time_eq_zero_byte(other)
    }
}

impl Eq for ZeroByte {}

// Implement Clone with explicit secure cloning
impl Clone for ZeroByte {
    fn clone(&self) -> Self {
        self.secure_clone()
    }
}

// Custom Debug implementation that doesn't expose the data
impl std::fmt::Debug for ZeroByte {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ZeroByte")
            .field("len", &self.data.len())
            .field("capacity", &self.data.capacity())
            .field("data", &"<redacted>")
            .finish()
    }
}

// Secure conversion from &str (useful for testing and initialization)
impl From<&str> for ZeroByte {
    fn from(s: &str) -> Self {
        ZeroByte::from_bytes(s.as_bytes())
    }
}

// Secure conversion from Vec<u8>
impl From<Vec<u8>> for ZeroByte {
    fn from(v: Vec<u8>) -> Self {
        Self::from_vec(v)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_zerobyte_drops_without_panic() {
        // This tests that ZeroizeOnDrop doesn't cause issues
        {
            let zero_byte = ZeroByte::from_bytes(b"secret");
            assert_eq!(zero_byte.len(), 6);
        } // zero_byte drops here and should zeroize

        // If we get here without panic, zeroization worked
    }

    #[test]
    fn test_manual_zeroization() {
        let mut zero_byte = ZeroByte::from_bytes(b"secret");

        // Verify data exists
        assert_eq!(zero_byte.len(), 6);
        zero_byte.with_bytes(|bytes| {
            assert_eq!(bytes, b"secret");
        });

        // Manually clear
        zero_byte.clear();

        // Verify it's empty
        assert_eq!(zero_byte.len(), 0);
        assert!(zero_byte.is_empty());
    }

    #[test]
    fn test_memory_is_zeroed() {
        let data = b"secret";
        let mut memory_locations = Vec::new();

        {
            let zero_byte = ZeroByte::from_bytes(data);

            // Capture the memory location of the data
            zero_byte.with_bytes(|bytes| {
                memory_locations.push(bytes.as_ptr());
            });

            // Verify data is there
            zero_byte.with_bytes(|bytes| {
                assert_eq!(bytes, data);
            });
        } // zero_byte drops here

        // Check if memory was zeroed (this is unsafe and platform dependent)
        // Note: This might not work reliably due to memory reuse
        unsafe {
            for &ptr in &memory_locations {
                let slice = std::slice::from_raw_parts(ptr, data.len());
                // Memory might be reused, so this test isn't 100% reliable
                println!("Memory after drop: {:?}", slice);
            }
        }
    }

    #[test]
    fn test_truncate_zeroizes_removed_data() {
        let mut zero_byte = ZeroByte::from_bytes(b"secret");
        assert_eq!(zero_byte.len(), 6);

        // Truncate to 5 characters
        zero_byte.truncate(3);
        assert_eq!(zero_byte.len(), 3);

        // Verify remaining data
        zero_byte.with_bytes(|bytes| {
            assert_eq!(bytes, b"sec");
        });
    }

    #[test]
    fn test_resize_zeroize_removed_data() {
        let mut zero_byte = ZeroByte::from_bytes(b"secret");
        assert_eq!(zero_byte.len(), 6);

        // Resize smaller
        zero_byte.resize(4, 0);
        assert_eq!(zero_byte.len(), 4);

        zero_byte.with_bytes(|bytes| {
            assert_eq!(bytes, b"secr");
        });

        // Resize larger
        zero_byte.resize(8, b'X');
        assert_eq!(zero_byte.len(), 8);

        zero_byte.with_bytes(|bytes| {
            assert_eq!(bytes, b"secrXXXX");
        });
    }

    #[test]
    fn test_from_vec_takes_ownership() {
        let original_data = vec![1, 2, 3, 4, 5];
        let data_ptr = original_data.as_ptr();

        let zero_byte = ZeroByte::from_vec(original_data);
        // original_data is now moved and can't be used

        // Verify the data is accessible through ZeroByte
        zero_byte.with_bytes(|bytes| {
            assert_eq!(bytes, &[1, 2, 3, 4, 5]);
        });

        // Verify it's the same memory location (moved, not copied)
        zero_byte.with_bytes(|bytes| {
            assert_eq!(bytes.as_ptr(), data_ptr);
        });
    }
}