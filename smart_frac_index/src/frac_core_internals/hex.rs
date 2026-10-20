use fracindex::Fracindex;

use crate::frac_core_internals::errors::{FracIndexError, err};

pub(crate) fn bytes_to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}

fn nibble(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

pub(crate) fn hex_to_bytes(hex: &str) -> Option<Vec<u8>> {
    if hex.len() % 2 != 0 {
        return None;
    }
    let raw = hex.as_bytes();
    let mut out = Vec::with_capacity(raw.len() / 2);
    let mut i = 0;
    while i < raw.len() {
        let hi = nibble(raw[i])?;
        let lo = nibble(raw[i + 1])?;
        out.push((hi << 4) | lo);
        i += 2;
    }
    Some(out)
}

pub(crate) fn parse_key(hex: &str, method: &str) -> Result<Fracindex, FracIndexError> {
    let bytes = hex_to_bytes(hex).ok_or_else(|| err(format!("not hex: {hex:?}"), method))?;
    Fracindex::from_bytes(&bytes).map_err(|e| err(format!("bad key {hex:?}: {e:?}"), method))
}
