use leptos::prelude::*;
use std::sync::LazyLock;

static SENDING: LazyLock<ArcRwSignal<bool>> =
    LazyLock::new(|| ArcRwSignal::new(false));

static LAST_SEND_FAILED: LazyLock<ArcRwSignal<bool>> =
    LazyLock::new(|| ArcRwSignal::new(false));

pub fn sending_signal() -> ArcRwSignal<bool> {
    SENDING.clone()
}

pub fn last_send_failed_signal() -> ArcRwSignal<bool> {
    LAST_SEND_FAILED.clone()
}

fn sending_is_allowed_now() -> bool {
    !crate::sync_mode::is_gated()
        && crate::server_link::conn_state::current_conn_state()
            == crate::server_link::conn_state::ConnState::Connected
}

fn report_send_failure(error: crate::client_server_coms::ComsError, queued: usize) {
    LAST_SEND_FAILED.set(true);
    if matches!(error, crate::client_server_coms::ComsError::Server(_)) {
        crate::cool_popup::show_popup_as(
            crate::cool_popup::PopupKind::Error,
            format!("Sending failed: {error}. {queued} edits still queued; they retry on the next edit or reconnect."),
        );
    }
}

async fn send_pending_letters_until_blocked(
    coms: crate::client_server_coms::ComsHandleForClient,
) {
    loop {
        if !sending_is_allowed_now() {
            break;
        }
        let letter = crate::server_link::pending_queue::front_pending_letter();
        let Some(letter) = letter else { break };
        match coms.send_letter(letter).await {
            Ok(()) => {
                LAST_SEND_FAILED.set(false);
                crate::server_link::pending_queue::drop_front_pending_letter();
            }
            Err(e) => {
                let n = crate::server_link::pending_queue::pending_count_signal().get_untracked();
                report_send_failure(e, n);
                break;
            }
        }
    }
    SENDING.set(false);
}

pub fn kick_send(coms: crate::client_server_coms::ComsHandleForClient) {
    if !sending_is_allowed_now() {
        return;
    }
    if SENDING.get_untracked() {
        return;
    }
    SENDING.set(true);
    leptos::task::spawn_local(async move {
        send_pending_letters_until_blocked(coms).await;
    });
}
