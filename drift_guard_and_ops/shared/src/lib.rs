//! Fingerprinting for page content, shared by the client and the server.
//!
//! Caller pov: `fingerprint(x)` gives a hex SHA-256 of `x`, where `x`
//! is a `&[String]` (ordered block texts) or `&[u8]` (a yrs snapshot).
//! Which types are supported lives in the `Hashable` impls; adding a
//! new fingerprinted type means adding one impl.
//!
//! Why not `DefaultHasher`: it isn't stable across rustc versions, and
//! the client and server are built separately. SHA-256 is standardised
//! and stable on every target.

use sha2::{Digest, Sha256};

/// Types that can be turned into the bytes that get fingerprinted.
/// Implement this for anything whose byte representation is a stable
/// page identity.
pub trait Hashable {
    fn to_hashable_bytes(&self) -> Vec<u8>;
}

impl Hashable for &[String] {
    fn to_hashable_bytes(&self) -> Vec<u8> {
        // Length-prefix each string and the list, so ["ab","c"] and
        // ["a","bc"] produce different byte streams.
        let mut out = Vec::new();
        out.extend_from_slice(&(self.len() as u64).to_le_bytes());
        for s in self.iter() {
            out.extend_from_slice(&(s.len() as u64).to_le_bytes());
            out.extend_from_slice(s.as_bytes());
        }
        out
    }
}

impl Hashable for &[u8] {
    fn to_hashable_bytes(&self) -> Vec<u8> {
        self.to_vec()
    }
}

/// SHA-256 hex of `x`'s hashed form.
///
/// Stable: same value gives the same hex on every target and every
/// rustc version. Changing what a `Hashable` impl writes is a protocol
/// break -- the client and server would stop agreeing on page identity.
pub fn fingerprint(x: impl Hashable) -> String {
    hex(&Sha256::digest(x.to_hashable_bytes()))
}

fn hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_answer_of_empty_bytes_is_sha256_of_empty() {
        // SHA-256 of "" is the standard empty-input constant.
        let empty: &[u8] = &[];
        assert_eq!(
            fingerprint(empty),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn ab_c_differs_from_a_bc() {
        let a = vec!["ab".to_string(), "c".to_string()];
        let b = vec!["a".to_string(), "bc".to_string()];
        assert_ne!(fingerprint(a.as_slice()), fingerprint(b.as_slice()));
    }

    #[test]
    fn empty_vec_differs_from_one_empty_string() {
        let a: Vec<String> = Vec::new();
        let b = vec![String::new()];
        assert_ne!(fingerprint(a.as_slice()), fingerprint(b.as_slice()));
    }

    #[test]
    fn same_input_same_output() {
        let v = vec!["hello".to_string(), "world".to_string()];
        assert_eq!(fingerprint(v.as_slice()), fingerprint(v.as_slice()));
    }
}
