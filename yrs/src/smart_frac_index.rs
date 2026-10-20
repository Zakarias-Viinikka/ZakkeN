//! Fractional indexing.
//!
//! A block's place in a page is a hex string that sorts by plain
//! string comparison. Two blocks compare by this value to decide
//! which comes first. A new place can always be made between two
//! existing places, so nothing ever has to be renumbered.
//!
//! `fracindex` does the work; this file is the only place that
//! talks to it. The crate works in raw bytes; hex is just a
//! string wrapper so the key fits next to `meta` in the block map.

use crate::yrs_error::{ErrorInfo, YrsError};
use fracindex::Fracindex;

fn err(msg: impl Into<String>, method: &str) -> YrsError {
    YrsError::GenericError {
        info: ErrorInfo {
            error_msg: msg.into(),
            file: file!().to_string(),
            method: method.to_string(),
        },
    }
}

fn bytes_to_hex(bytes: &[u8]) -> String {
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

fn hex_to_bytes(hex: &str) -> Option<Vec<u8>> {
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

fn parse_key(hex: &str, method: &str) -> Result<Fracindex, YrsError> {
    let bytes = hex_to_bytes(hex)
        .ok_or_else(|| err(format!("smart_frac_index: not hex: {hex:?}"), method))?;
    Fracindex::from_bytes(&bytes)
        .map_err(|e| err(format!("smart_frac_index: bad key {hex:?}: {e:?}"), method))
}

/// The place a block gets when it is the first block on a page.
pub fn key_at_start() -> String {
    bytes_to_hex(&Fracindex::default().to_bytes())
}

/// A place that comes after `key`.
pub fn key_after(key: &str) -> Result<String, YrsError> {
    let k = parse_key(key, "key_after")?;
    let next = Fracindex::builder()
        .after(&k)
        .build()
        .map_err(|e| err(format!("smart_frac_index: after(): {e:?}"), "key_after"))?;
    Ok(bytes_to_hex(&next.to_bytes()))
}

/// A place that comes before `key`.
pub fn key_before(key: &str) -> Result<String, YrsError> {
    let k = parse_key(key, "key_before")?;
    let prev = Fracindex::builder()
        .before(&k)
        .build()
        .map_err(|e| err(format!("smart_frac_index: before(): {e:?}"), "key_before"))?;
    Ok(bytes_to_hex(&prev.to_bytes()))
}

/// A place that comes after `a` and before `b`.
pub fn key_between(a: &str, b: &str) -> Result<String, YrsError> {
    let pa = parse_key(a, "key_between")?;
    let pb = parse_key(b, "key_between")?;
    let mid = Fracindex::builder()
        .between(&pa, &pb)
        .build()
        .map_err(|e| {
            err(
                format!("smart_frac_index: no room between {a:?} and {b:?}: {e:?}"),
                "key_between",
            )
        })?;
    Ok(bytes_to_hex(&mid.to_bytes()))
}