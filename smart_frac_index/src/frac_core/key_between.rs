use crate::frac_core_internals::errors::{FracIndexError, err};
use crate::frac_core_internals::hex::{bytes_to_hex, parse_key};
use fracindex::Fracindex;

/// A place that comes after `a` and before `b`.
pub fn key_between(a: &str, b: &str) -> Result<String, FracIndexError> {
    let pa = parse_key(a, "key_between")?;
    let pb = parse_key(b, "key_between")?;
    let mid = Fracindex::builder()
        .between(&pa, &pb)
        .build()
        .map_err(|e| err(format!("no room between {a:?} and {b:?}: {e:?}"), "key_between"))?;
    Ok(bytes_to_hex(&mid.to_bytes()))
}
