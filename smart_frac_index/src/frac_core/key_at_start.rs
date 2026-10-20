use crate::frac_core_internals::hex::bytes_to_hex;
use fracindex::Fracindex;

/// The place a value gets when it is the first in an ordered list.
pub fn key_at_start() -> String {
    bytes_to_hex(&Fracindex::default().to_bytes())
}
