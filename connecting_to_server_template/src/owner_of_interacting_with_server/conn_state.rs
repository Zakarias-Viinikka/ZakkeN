use leptos::prelude::*;
use std::sync::LazyLock;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ConnState {
    Offline,
    Connecting,
    FailedToConnect,
    Connected,
}

static CONN_STATE: LazyLock<ArcRwSignal<ConnState>> =
    LazyLock::new(|| ArcRwSignal::new(ConnState::Offline));

pub fn conn_state_signal() -> ArcRwSignal<ConnState> {
    CONN_STATE.clone()
}

pub fn current_conn_state() -> ConnState {
    CONN_STATE.get_untracked()
}

pub fn require_server() -> Result<i64, String> {
    if CONN_STATE.get_untracked() != ConnState::Connected {
        return Err("server not connected".into());
    }
    match crate::owner_of_interacting_with_server::dataset_generation::current_generation() {
        Some(g) => Ok(g),
        None => Err("not logged in yet".into()),
    }
}

/// Lonewolf: socket is up and login succeeded, but sync is gated.
pub fn is_lonewolf() -> bool {
    current_conn_state() == ConnState::Connected && crate::sync_mode::is_gated()
}

/// Go offline without a popup. Used when another message already explained
/// why (a DatasetReplaced arrived and popped its own warning) and firing a
/// second "connection lost" popup would just be noise.
pub fn force_offline() {
    CONN_STATE.set(ConnState::Offline);
    crate::owner_of_interacting_with_server::sending::sending_signal().set(false);
}

pub fn go_offline_after_connection_lost() {
    if crate::owner_of_interacting_with_server::restart::restart_in_flight_signal().get_untracked() {
        return;
    }
    let was_connected = CONN_STATE.get_untracked() == ConnState::Connected;
    force_offline();
    if was_connected {
        crate::cool_popup::show_popup_as(
            crate::cool_popup::PopupKind::Warning,
            "Connection to the server was lost",
        );
    }
}
