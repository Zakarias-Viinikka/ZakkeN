//! Two-state sync mode. Paused was merged into NeedsReconcile: pausing now
//! means "stop syncing, you must Reconcile or Adopt to get back." Persisted
//! across reloads (mode + reason).
//!
//! Lonewolf = the socket is up and login succeeded but sync is gated.
//! ConnState::Connected with SyncMode::NeedsReconcile. Distinct from Offline
//! (socket down) and Connected (syncing normally). Not a SyncMode variant —
//! see `server_link::is_lonewolf` and `client::BorderState::Lonewolf`.

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

const PERSIST_MODE_KEY: &str = "template_sync_mode";
const PERSIST_REASON_KEY: &str = "template_mode_reason";

fn storage() -> Option<web_sys::Storage> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
}

fn read_persisted() -> (SyncMode, Option<ModeReason>) {
    let Some(s) = storage() else {
        return (SyncMode::Connected, None);
    };
    let mode = s
        .get_item(PERSIST_MODE_KEY)
        .ok()
        .flatten()
        .and_then(|v| match v.as_str() {
            "needs_reconcile" => Some(SyncMode::NeedsReconcile),
            _ => None,
        })
        .unwrap_or(SyncMode::Connected);
    let reason = if mode == SyncMode::NeedsReconcile {
        s.get_item(PERSIST_REASON_KEY)
            .ok()
            .flatten()
            .and_then(|v| match v.as_str() {
                "you_paused" => Some(ModeReason::YouPaused),
                "you_wiped" => Some(ModeReason::YouWiped),
                "server_changed" => Some(ModeReason::ServerChanged),
                "wipe_incomplete" => Some(ModeReason::WipeIncomplete),
                "adopt_incomplete" => Some(ModeReason::AdoptIncomplete),
                "reconciling" => Some(ModeReason::Reconciling),
                "reconcile_incomplete" => Some(ModeReason::ReconcileIncomplete),
                _ => None,
            })
    } else {
        None
    };
    (mode, reason)
}

fn reason_key(r: ModeReason) -> &'static str {
    match r {
        ModeReason::YouPaused => "you_paused",
        ModeReason::YouWiped => "you_wiped",
        ModeReason::ServerChanged => "server_changed",
        ModeReason::WipeIncomplete => "wipe_incomplete",
        ModeReason::AdoptIncomplete => "adopt_incomplete",
        ModeReason::Reconciling => "reconciling",
        ModeReason::ReconcileIncomplete => "reconcile_incomplete",
    }
}

fn persist_state() {
    let Some(s) = storage() else { return };
    match SYNC_MODE.get_untracked() {
        SyncMode::NeedsReconcile => {
            let _ = s.set_item(PERSIST_MODE_KEY, "needs_reconcile");
            if let Some(r) = MODE_REASON.get_untracked() {
                let _ = s.set_item(PERSIST_REASON_KEY, reason_key(r));
            } else {
                let _ = s.remove_item(PERSIST_REASON_KEY);
            }
        }
        SyncMode::Connected => {
            let _ = s.remove_item(PERSIST_MODE_KEY);
            let _ = s.remove_item(PERSIST_REASON_KEY);
        }
    }
}

static SYNC_MODE: LazyLock<ArcRwSignal<SyncMode>> =
    LazyLock::new(|| ArcRwSignal::new(read_persisted().0));

static MODE_REASON: LazyLock<ArcRwSignal<Option<ModeReason>>> =
    LazyLock::new(|| ArcRwSignal::new(read_persisted().1));

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
    persist_state();
}

pub fn current_mode_reason() -> Option<ModeReason> {
    MODE_REASON.get_untracked()
}

pub fn mode_reason_signal() -> ArcRwSignal<Option<ModeReason>> {
    MODE_REASON.clone()
}

pub fn set_mode_reason(reason: ModeReason) {
    MODE_REASON.set(Some(reason));
    persist_state();
}

pub fn clear_mode_reason() {
    MODE_REASON.set(None);
    persist_state();
}

pub fn sync_mode_signal() -> ArcRwSignal<SyncMode> {
    SYNC_MODE.clone()
}