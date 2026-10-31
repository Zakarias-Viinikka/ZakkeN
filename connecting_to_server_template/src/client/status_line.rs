use leptos::prelude::*;

use crate::shared_owners::client_status::{client_status_signal, ClientStatus};

pub fn status_line() -> Signal<String> {
    let status = client_status_signal();
    Signal::derive(move || {
        let sending = crate::owner_of_interacting_with_server::sending::sending_signal().get();
        let last_failed =
            crate::owner_of_interacting_with_server::sending::last_send_failed_signal().get();
        let pending =
            crate::owner_of_interacting_with_server::pending_queue::pending_count_signal().get();
        let plan_open = crate::reconcile::open_signal().get();
        let plan_error = crate::reconcile::error_signal().get().is_some();
        let reason = crate::sync_mode::mode_reason_signal().get();

        if plan_open {
            return status_while_reconcile_modal_is_open(plan_error);
        }
        match status.get() {
            ClientStatus::NotStarted => "Not started.".into(),
            ClientStatus::Locked => "Locked. Reload the page.".into(),
            ClientStatus::NeedsReconcile | ClientStatus::Lonewolf => {
                status_while_sync_is_stopped(status.get(), reason)
            }
            other => status_while_sync_is_running(other, sending, last_failed, pending),
        }
    })
}

pub fn reason_label(reason: Option<crate::sync_mode::ModeReason>) -> &'static str {
    use crate::sync_mode::ModeReason::*;
    match reason {
        Some(YouPaused) => "you paused",
        Some(YouWiped) => "you wiped",
        Some(ServerChanged) => "server changed",
        Some(WipeIncomplete) => "wipe incomplete",
        Some(AdoptIncomplete) => "adopt incomplete",
        Some(Reconciling) => "reconciling",
        Some(ReconcileIncomplete) => "reconcile incomplete",
        None => "unknown",
    }
}

fn status_while_reconcile_modal_is_open(plan_error: bool) -> String {
    let suffix = if plan_error { " Last apply failed." } else { "" };
    format!("Reconcile in progress, sync stopped.{suffix}")
}

fn status_while_sync_is_stopped(
    status: ClientStatus,
    reason: Option<crate::sync_mode::ModeReason>,
) -> String {
    let (presence, connected) = if status == ClientStatus::Lonewolf {
        ("Lonewolf", "(connected)")
    } else {
        ("Stopped", "(offline)")
    };
    format!(
        "{presence}: {}. Reconcile or Adopt. {connected}",
        reason_label(reason)
    )
}

fn status_while_sync_is_running(
    status: ClientStatus,
    sending: bool,
    last_failed: bool,
    pending: usize,
) -> String {
    match status {
        ClientStatus::Connecting => "Connecting…".into(),
        ClientStatus::FailedToConnect => format!("Couldn't connect. {pending} waiting."),
        ClientStatus::Offline => format!("Not connected. {pending} edits waiting."),
        ClientStatus::Connected => {
            if sending {
                format!("Sending {pending}…")
            } else if last_failed {
                format!("{pending} waiting: last send failed.")
            } else if pending == 0 {
                "In sync.".into()
            } else {
                format!("{pending} edits waiting.")
            }
        }
        _ => format!("{status:?}"),
    }
}