use crate::frac_core_internals::errors::{FracIndexError, err};
use crate::frac_core_internals::hex::{bytes_to_hex, parse_key};
use fracindex::Fracindex;

/// A place that comes before `key`.
pub fn key_before(key: &str) -> Result<String, FracIndexError> {
    let k = parse_key(key, "key_before")?;
    let prev = Fracindex::builder()
        .before(&k)
        .build()
        .map_err(|e| err(format!("before(): {e:?}"), "key_before"))?;
    Ok(bytes_to_hex(&prev.to_bytes()))
}
