use crate::frac_core_internals::errors::{FracIndexError, err};
use crate::frac_core_internals::hex::{bytes_to_hex, parse_key};
use fracindex::Fracindex;

/// A place that comes after `key`.
pub fn key_after(key: &str) -> Result<String, FracIndexError> {
    let k = parse_key(key, "key_after")?;
    let next = Fracindex::builder()
        .after(&k)
        .build()
        .map_err(|e| err(format!("after(): {e:?}"), "key_after"))?;
    Ok(bytes_to_hex(&next.to_bytes()))
}
