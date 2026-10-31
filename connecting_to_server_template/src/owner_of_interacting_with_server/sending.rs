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
    !crate::owner_of_interacting_with_server::locked::is_locked()
        && !crate::sync_mode::is_gated()
        && crate::owner_of_interacting_with_server::conn_state::current_conn_state()
            == crate::owner_of_interacting_with_server::conn_state::ConnState::Connected
}

enum SendResult {
    Sent,
    TransportDied,
    LockNow,
}

// One attempt plus one retry, but only for server errors. Transport
// errors mean the socket died; the offline retry loop owns that path
// and this send should just stop. Two server rejections of the same
// letter is the trigger for the hard lock: sync is stuck in a way that
// no retry will fix, so the only way out is a reload.
async fn send_one_letter_with_retry(
    coms: crate::client_server_coms::ComsHandleForClient,
    letter: crate::protocol::LoveLetter,
) -> SendResult {
    use crate::client_server_coms::ComsError;
    match coms.send_letter(letter.clone()).await {
        Ok(()) => SendResult::Sent,
        Err(ComsError::Transport(_)) => SendResult::TransportDied,
        Err(ComsError::Server(first)) => {
            crate::cool_popup::show_popup_as(
                crate::cool_popup::PopupKind::Warning,
                format!("Trying to send again… ({first})"),
            );
            if !sending_is_allowed_now() {
                return SendResult::TransportDied;
            }
            match coms.send_letter(letter).await {
                Ok(()) => SendResult::Sent,
                Err(ComsError::Transport(_)) => SendResult::TransportDied,
                Err(ComsError::Server(_)) => SendResult::LockNow,
            }
        }
    }
}

async fn send_pending_letters_until_blocked(
    coms: crate::client_server_coms::ComsHandleForClient,
) {
    loop {
        if !sending_is_allowed_now() {
            break;
        }
        let Some(letter) =
            crate::owner_of_interacting_with_server::pending_queue::front_pending_letter()
        else {
            break;
        };
        match send_one_letter_with_retry(coms.clone(), letter).await {
            SendResult::Sent => {
                LAST_SEND_FAILED.set(false);
                crate::owner_of_interacting_with_server::pending_queue::drop_front_pending_letter();
            }
            SendResult::TransportDied => break,
            SendResult::LockNow => {
                LAST_SEND_FAILED.set(true);
                crate::owner_of_interacting_with_server::locked::lock_and_announce();
                break;
            }
        }
    }
    SENDING.set(false);
}

pub fn main_send_unsent_love_letters(coms: crate::client_server_coms::ComsHandleForClient) {
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