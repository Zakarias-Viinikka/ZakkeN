//! Android-facing wrappers around `frac_core`.
//!
//! Thin — each one calls the real function in `frac_core` and lets
//! its `FracIndexError` through unchanged. Nothing else here.

use crate::frac_core::{
    key_after as real_key_after, key_at_start as real_key_at_start,
    key_before as real_key_before, key_between as real_key_between,
};
use crate::frac_core_internals::errors::FracIndexError;

#[uniffi::export]
pub fn key_at_start() -> String {
    real_key_at_start()
}

#[uniffi::export]
pub fn key_after(key: String) -> Result<String, FracIndexError> {
    real_key_after(&key)
}

#[uniffi::export]
pub fn key_before(key: String) -> Result<String, FracIndexError> {
    real_key_before(&key)
}

#[uniffi::export]
pub fn key_between(a: String, b: String) -> Result<String, FracIndexError> {
    real_key_between(&a, &b)
}