//! The client's current state, computed once and shared.
//!
//! Every place that needs to know "what state is the client in" reads
//! this signal instead of re-deriving from ConnState + SyncMode + locked
//! + client_started every time.
//!
//! Caller pov: `client_status_signal().get()` gives you the current
//! ClientStatus. Reactive — subscribes to the underlying signals.
//!
//! Internals pov: derived from four signals:
//! - `client::client_started_signal()` — flipped by the Start client
//!   button.
//! - `owner_of_interacting_with_server::locked::hard_locked_signal()` —
//!   set once when two sends fail in a row.
//! - `owner_of_interacting_with_server::conn_state::conn_state_signal()` —
//!   set by restart / disconnect paths.
//! - `sync_mode::sync_mode_signal()` — set by pause / wipe / adopt /
//!   reconcile / login generation check.
//!
//! Readers: `client::components::border` (border colour),
//! `client::status_line` (status text), `app_main::client_title`
//! (title), `app_main::main_for_retrying_the_connection_while_offline`
//! (retry gate).

use leptos::prelude::*;
use std::sync::LazyLock;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ClientStatus {
    /// The user hasn't pressed Start client yet.
    NotStarted,
    /// Hard lock. Only a reload exits this state.
    Locked,
    Offline,
    Connecting,
    FailedToConnect,
    Connected,
    NeedsReconcile,
    Lonewolf,
}

static CLIENT_STATUS: LazyLock<Signal<ClientStatus>> = LazyLock::new(|| {
    Signal::derive(|| {
        if !crate::client::client_started_signal().get() {
            return ClientStatus::NotStarted;
        }
        if crate::owner_of_interacting_with_server::locked::hard_locked_signal().get() {
            return ClientStatus::Locked;
        }
        use crate::owner_of_interacting_with_server::conn_state::{
            conn_state_signal, ConnState,
        };
        use crate::sync_mode::{sync_mode_signal, SyncMode};
        let conn = conn_state_signal().get();
        let mode = sync_mode_signal().get();
        match (conn, mode) {
            (_, SyncMode::NeedsReconcile) => match conn {
                ConnState::Connected => ClientStatus::Lonewolf,
                _ => ClientStatus::NeedsReconcile,
            },
            (ConnState::Connected, SyncMode::Connected) => ClientStatus::Connected,
            (ConnState::Connecting, _) => ClientStatus::Connecting,
            (ConnState::FailedToConnect, _) => ClientStatus::FailedToConnect,
            (ConnState::Offline, _) => ClientStatus::Offline,
        }
    })
});

pub fn client_status_signal() -> Signal<ClientStatus> {
    CLIENT_STATUS.clone()
}
