//! Two-state sync mode. Paused was merged into NeedsReconcile: pausing now
//! means "stop syncing, you must Reconcile or Adopt to get back." Persisted
//! across reloads (mode + reason).
//!
//! Lonewolf = the socket is up and login succeeded but sync is gated.
//! ConnState::Connected with SyncMode::NeedsReconcile. Distinct from Offline
//! (socket down) and Connected (syncing normally). Not a SyncMode variant —
//! see `owner_of_interacting_with_server::conn_state::is_lonewolf` and
//! `client::border::BorderState::Lonewolf`.

pub mod persistence;

use leptos::prelude::*;
use std::sync::LazyLock;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SyncMode {
    Connected,
    NeedsReconcile,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ModeReason {
    YouPaused,
    YouWiped,
    ServerChanged,
    WipeIncomplete,
    AdoptIncomplete,
    Reconciling,
    ReconcileIncomplete,
}

static SYNC_MODE: LazyLock<ArcRwSignal<SyncMode>> =
    LazyLock::new(|| ArcRwSignal::new(persistence::read_persisted().0));

static MODE_REASON: LazyLock<ArcRwSignal<Option<ModeReason>>> =
    LazyLock::new(|| ArcRwSignal::new(persistence::read_persisted().1));

pub fn current_mode() -> SyncMode {
    SYNC_MODE.get_untracked()
}

pub fn is_gated() -> bool {
    current_mode() != SyncMode::Connected
}

pub fn set_sync_mode(mode: SyncMode) {
    let current = SYNC_MODE.get_untracked();
    if current == SyncMode::NeedsReconcile && mode != SyncMode::NeedsReconcile {
        MODE_REASON.set(None);
    }
    SYNC_MODE.set(mode);
    persistence::persist_state();
}

pub fn current_mode_reason() -> Option<ModeReason> {
    MODE_REASON.get_untracked()
}

pub fn mode_reason_signal() -> ArcRwSignal<Option<ModeReason>> {
    MODE_REASON.clone()
}

pub fn set_mode_reason(reason: ModeReason) {
    MODE_REASON.set(Some(reason));
    persistence::persist_state();
}

pub fn clear_mode_reason() {
    MODE_REASON.set(None);
    persistence::persist_state();
}

pub fn sync_mode_signal() -> ArcRwSignal<SyncMode> {
    SYNC_MODE.clone()
}