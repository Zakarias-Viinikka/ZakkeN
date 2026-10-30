use leptos::prelude::*;

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
    conn: crate::server_link::conn_state::ConnState,
    reason: Option<crate::sync_mode::ModeReason>,
) -> String {
    use crate::server_link::conn_state::ConnState;
    let connected = if conn == ConnState::Connected {
        "(connected)"
    } else {
        "(offline)"
    };
    let presence = if conn == ConnState::Connected {
        "Lonewolf"
    } else {
        "Stopped"
    };
    format!(
        "{presence}: {}. Reconcile or Adopt. {connected}",
        reason_label(reason)
    )
}

fn status_while_sync_is_running(
    conn: crate::server_link::conn_state::ConnState,
    sending: bool,
    last_failed: bool,
    pending: usize,
) -> String {
    use crate::server_link::conn_state::ConnState;
    match conn {
        ConnState::Connecting => "Connecting…".into(),
        ConnState::FailedToConnect => format!("Couldn't connect. {pending} waiting."),
        ConnState::Offline => format!("Not connected. {pending} edits waiting."),
        ConnState::Connected => {
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
    }
}

pub fn status_line() -> Signal<String> {
    Signal::derive(move || {
        use crate::sync_mode::SyncMode;
        let mode = crate::sync_mode::sync_mode_signal().get();
        let conn = crate::server_link::conn_state::conn_state_signal().get();
        let sending = crate::server_link::sending::sending_signal().get();
        let last_failed = crate::server_link::sending::last_send_failed_signal().get();
        let pending = crate::server_link::pending_queue::pending_count_signal().get();
        let plan_open = crate::reconcile::plan_signal().get().is_some();
        let plan_error = crate::reconcile::error_signal().get().is_some();
        let reason = crate::sync_mode::mode_reason_signal().get();

        if plan_open {
            return status_while_reconcile_modal_is_open(plan_error);
        }
        match mode {
            SyncMode::NeedsReconcile => status_while_sync_is_stopped(conn, reason),
            SyncMode::Connected => {
                status_while_sync_is_running(conn, sending, last_failed, pending)
            }
        }
    })
}
