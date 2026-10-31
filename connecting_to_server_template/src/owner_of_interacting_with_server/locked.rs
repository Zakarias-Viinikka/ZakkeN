//! One-way "stop everything until restart" state. Triggered by two
//! consecutive server rejections of the same send while the socket is
//! still up. Distinct from SyncMode::NeedsReconcile (recoverable via
//! Reconcile/Adopt) and ConnState::Offline (recoverable via the offline
//! retry loop). Locked only exits on reload.

use leptos::prelude::*;
use std::sync::LazyLock;

static HARD_LOCKED: LazyLock<ArcRwSignal<bool>> =
    LazyLock::new(|| ArcRwSignal::new(false));

pub fn hard_locked_signal() -> ArcRwSignal<bool> {
    HARD_LOCKED.clone()
}

pub fn is_locked() -> bool {
    HARD_LOCKED.get_untracked()
}

pub fn lock_and_announce() {
    if is_locked() {
        return;
    }
    HARD_LOCKED.set(true);
    crate::owner_of_interacting_with_server::conn_state::force_offline();
    let n = crate::owner_of_interacting_with_server::pending_queue::pending_count_signal()
        .get_untracked();
    crate::cool_popup::show_popup_as(
        crate::cool_popup::PopupKind::Error,
        format!(
            "Send failed twice while online. Sync is locked: nothing will be sent or received until you restart. {n} queued edit(s) are stuck in the outbox — copy anything you're worried about losing, then reload the page."
        ),
    );
}
