use std::sync::atomic::{AtomicU64, Ordering};

use rand::prelude::*;

pub(crate) const BLOCKS_KEY: &str = "blocks";
pub(crate) const CONTENT_KEY: &str = "text";
pub(crate) const META_KEY: &str = "meta";
pub(crate) const ID_KEY: &str = "id";
pub(crate) const FRAC_POSITION_KEY: &str = "frac_position";

static COUNTER: AtomicU64 = AtomicU64::new(0);

#[uniffi::export]
pub fn generate_unique_key(user_id: &str, time: &str) -> String {
    let unique_time = format!("{}-{}", time, COUNTER.fetch_add(1, Ordering::SeqCst));
    let mut rng = rand::rng();
    let random_part: u64 = rng.random();
    format!("{unique_time}-{random_part}-{user_id}")
}
