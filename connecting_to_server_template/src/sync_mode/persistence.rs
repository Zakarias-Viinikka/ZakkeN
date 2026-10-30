use crate::sync_mode::{ModeReason, SyncMode};

const PERSIST_MODE_KEY: &str = "template_sync_mode";
const PERSIST_REASON_KEY: &str = "template_mode_reason";

fn storage() -> Option<web_sys::Storage> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
}

fn mode_from_stored_text(text: Option<String>) -> SyncMode {
    match text.as_deref() {
        Some("needs_reconcile") => SyncMode::NeedsReconcile,
        _ => SyncMode::Connected,
    }
}

fn reason_from_stored_text(text: Option<String>) -> Option<ModeReason> {
    match text.as_deref() {
        Some("you_paused") => Some(ModeReason::YouPaused),
        Some("you_wiped") => Some(ModeReason::YouWiped),
        Some("server_changed") => Some(ModeReason::ServerChanged),
        Some("wipe_incomplete") => Some(ModeReason::WipeIncomplete),
        Some("adopt_incomplete") => Some(ModeReason::AdoptIncomplete),
        Some("reconciling") => Some(ModeReason::Reconciling),
        Some("reconcile_incomplete") => Some(ModeReason::ReconcileIncomplete),
        _ => None,
    }
}

fn reason_to_stored_text(reason: ModeReason) -> &'static str {
    match reason {
        ModeReason::YouPaused => "you_paused",
        ModeReason::YouWiped => "you_wiped",
        ModeReason::ServerChanged => "server_changed",
        ModeReason::WipeIncomplete => "wipe_incomplete",
        ModeReason::AdoptIncomplete => "adopt_incomplete",
        ModeReason::Reconciling => "reconciling",
        ModeReason::ReconcileIncomplete => "reconcile_incomplete",
    }
}

pub fn read_persisted() -> (SyncMode, Option<ModeReason>) {
    let Some(s) = storage() else {
        return (SyncMode::Connected, None);
    };
    let mode = mode_from_stored_text(s.get_item(PERSIST_MODE_KEY).ok().flatten());
    let reason = if mode == SyncMode::NeedsReconcile {
        reason_from_stored_text(s.get_item(PERSIST_REASON_KEY).ok().flatten())
    } else {
        None
    };
    (mode, reason)
}

pub fn persist_state() {
    let Some(s) = storage() else { return };
    match crate::sync_mode::current_mode() {
        SyncMode::NeedsReconcile => {
            let _ = s.set_item(PERSIST_MODE_KEY, "needs_reconcile");
            if let Some(r) = crate::sync_mode::current_mode_reason() {
                let _ = s.set_item(PERSIST_REASON_KEY, reason_to_stored_text(r));
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
