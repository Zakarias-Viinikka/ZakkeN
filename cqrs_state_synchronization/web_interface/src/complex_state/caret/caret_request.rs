//! Complex state: the ONE pending caret request ("put the caret in block X at place P").
//!
//! Who touches it:
//! - `complex_state/caret/focus.rs` (`at_end`, `at_start`, `at_pos`): writes a new request through
//!   `plexset_caret_request`. Called by the callbacks in `leptos_components/main_content/callbacks/*`
//!   after they change the rows. A new request overwrites the old one.
//! - `leptos_components/main_content/caret_owner.rs` (`CaretOwner`): reads it (tracked, in an
//!   Effect), and clears it with `plexset_clear_caret_request_if_serial` once the block's handler
//!   has run, or after about 2 s if nobody handled it.
//!
//! Every request gets a serial so "clear only if it is still the same request" is possible.
//! Offsets in `CaretPlace::At` are UTF-16 code units (what the editor uses).
use std::sync::LazyLock;
use std::sync::atomic::{AtomicU64, Ordering};

use leptos::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CaretPlace {
    Start,
    End,
    /// UTF-16 code unit offset.
    At(usize),
}

#[derive(Clone, Debug)]
pub struct CaretRequest {
    pub serial: u64,
    pub block_id: String,
    pub place: CaretPlace,
}

static CARET_REQUEST: LazyLock<ArcRwSignal<Option<CaretRequest>>> =
    LazyLock::new(|| ArcRwSignal::new(None));

static NEXT_SERIAL: AtomicU64 = AtomicU64::new(1);

pub fn plexget_caret_request() -> Option<CaretRequest> {
    CARET_REQUEST.get()
}

pub fn plexget_caret_request_untracked() -> Option<CaretRequest> {
    CARET_REQUEST.get_untracked()
}

/// Replaces any pending request.
pub fn plexset_caret_request(block_id: String, place: CaretPlace) {
    let serial = NEXT_SERIAL.fetch_add(1, Ordering::Relaxed);
    CARET_REQUEST.set(Some(CaretRequest {
        serial,
        block_id,
        place,
    }));
}

/// Clears the request only if it is still the one with this serial.
pub fn plexset_clear_caret_request_if_serial(serial: u64) {
    let is_still_it = CARET_REQUEST
        .with_untracked(|r| r.as_ref().map(|r| r.serial) == Some(serial));
    if is_still_it {
        CARET_REQUEST.set(None);
    }
}
