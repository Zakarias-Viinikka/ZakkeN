//! Reconcile verbs and modal state. The real flow is a process, not a
//! precomputed plan: the client hands the server its local pages, the
//! server checks each one and sends its own version when it doesn't
//! match. This template mocks that — no plan object, no decision map.
//! See `plan_for_general_data_management_ops.md` and
//! `behaviour_decisions.md` ("Not decisions" — reconcile).

use crate::client_server_coms::ComsHandleForClient;
use crate::sync_mode::{self, ModeReason, SyncMode};
use leptos::prelude::*;
use std::sync::{LazyLock, Mutex};

static MODAL_OPEN: LazyLock<ArcRwSignal<bool>> = LazyLock::new(|| ArcRwSignal::new(false));
static MODAL_ERROR: LazyLock<ArcRwSignal<Option<String>>> =
    LazyLock::new(|| ArcRwSignal::new(None));
static PREVIOUS_MODE_AND_REASON: LazyLock<Mutex<Option<(SyncMode, Option<ModeReason>)>>> =
    LazyLock::new(|| Mutex::new(None));

/// Entry from the Reconcile button. Enters `Reconciling`, stores the
/// mode/reason to restore on cancel, and opens the modal. The modal
/// reads `local_pages` directly; nothing else is carried between layers.
pub async fn reconcile_plan(
    _coms: ComsHandleForClient,
) -> Result<(), crate::ui_ops::verb_error::VerbError> {
    use crate::ui_ops::verb_error::VerbError;
    if MODAL_OPEN.get_untracked() {
        crate::cool_popup::show_popup("Reconcile already in progress.");
        return Ok(());
    }
    if crate::owner_of_interacting_with_server::conn_state::current_conn_state()
        != crate::owner_of_interacting_with_server::conn_state::ConnState::Connected
    {
        return Err(VerbError::refused(
            "Can't reconcile: not connected to the server",
        ));
    }
    let previous_mode = sync_mode::current_mode();
    let previous_reason = sync_mode::current_mode_reason();
    *PREVIOUS_MODE_AND_REASON.lock().expect("mode-and-reason poisoned") =
        Some((previous_mode, previous_reason));
    sync_mode::set_sync_mode(SyncMode::NeedsReconcile);
    sync_mode::set_mode_reason(ModeReason::Reconciling);
    MODAL_ERROR.set(None);
    MODAL_OPEN.set(true);
    Ok(())
}

/// Mock apply. The real flow runs the page-by-page process here,
/// rebuilding the local mirror from the server's pages inside one
/// transaction. The template just closes the modal, flips sync back on,
/// and pops a success.
pub async fn reconcile_apply(
    coms: ComsHandleForClient,
) -> Result<(), crate::ui_ops::verb_error::VerbError> {
    *PREVIOUS_MODE_AND_REASON.lock().expect("mode-and-reason poisoned") = None;
    sync_mode::set_sync_mode(SyncMode::Connected);
    clear_modal();
    let _ = crate::owner_of_interacting_with_server::restart::restart_sync_stack(coms).await;
    crate::cool_popup::show_popup_as(
        crate::cool_popup::PopupKind::Success,
        "Reconciled (mock). Sync resumed.",
    );
    Ok(())
}

/// Cancel: restore the mode/reason we had before opening the modal.
pub async fn reconcile_cancel(
    coms: ComsHandleForClient,
) -> Result<(), crate::ui_ops::verb_error::VerbError> {
    let restore = PREVIOUS_MODE_AND_REASON
        .lock()
        .expect("mode-and-reason poisoned")
        .take();
    if let Some((mode, reason)) = restore {
        sync_mode::set_sync_mode(mode);
        if let Some(r) = reason {
            sync_mode::set_mode_reason(r);
        }
    }
    clear_modal();
    let _ = crate::owner_of_interacting_with_server::restart::restart_sync_stack(coms).await;
    Ok(())
}

pub fn open_signal() -> ArcRwSignal<bool> {
    MODAL_OPEN.clone()
}

pub fn error_signal() -> ArcRwSignal<Option<String>> {
    MODAL_ERROR.clone()
}

pub fn clear_modal() {
    MODAL_OPEN.set(false);
    MODAL_ERROR.set(None);
}

pub fn apply_from_modal(coms: ComsHandleForClient) {
    leptos::task::spawn_local(async move {
        if let Err(e) = reconcile_apply(coms).await {
            MODAL_ERROR.set(Some(e.to_string()));
        }
    });
}

pub fn cancel_from_modal(coms: ComsHandleForClient) {
    leptos::task::spawn_local(async move {
        let _ = reconcile_cancel(coms).await;
    });
}