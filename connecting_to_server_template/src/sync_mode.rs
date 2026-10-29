//! Three-state sync mode. Mirrors the real `SYNC_MODE` in
//! `cqrs_state_synchronization/web_interface`. `is_gated()` is what the
//! four gating points read.

use leptos::prelude::*;
use std::sync::LazyLock;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SyncMode {
    Connected,
    Paused,
    NeedsReconcile,
}

static SYNC_MODE: LazyLock<ArcRwSignal<SyncMode>> =
    LazyLock::new(|| ArcRwSignal::new(SyncMode::Connected));

pub fn current_mode() -> SyncMode {
    SYNC_MODE.get_untracked()
}

pub fn is_gated() -> bool {
    current_mode() != SyncMode::Connected
}

// PLACEHOLDER: the real `set_sync_mode` writes to `key_value_storage` first,
// then updates the cache. Here it only updates the cache; there is no
// storage in the template.
pub fn set_sync_mode(mode: SyncMode) {
    SYNC_MODE.set(mode);
}

pub fn sync_mode_signal() -> ArcRwSignal<SyncMode> {
    SYNC_MODE.clone()
}
