//! Which of the client's connection states is current.
//!
//! The static signal lives here. Every reader and writer goes through
//! `plexget_conn_state()` / `plexget_conn_state_untracked()` /
//! `plexset_conn_state()` so a reader can see every toucher in one place.
//!
//! `plexget_conn_state()` calls `.get()` — for view closures. It
//! re-runs when the state changes. `plexget_conn_state_untracked()`
//! calls `.get_untracked()` — for non-reactive code where subscribing
//! would be wrong.
//!
//! Getters (literal path from the web_interface `src/` root):
//! - `owner_of_interacting_with_server/core.rs` — `sync_may_run`,
//!   untracked.
//! - `owner_of_interacting_with_server/drift/enter_lonewolf.rs` —
//!   untracked, picks the next state.
//! - `owner_of_interacting_with_server/main_for_login_and_registration.rs`
//!   — untracked.
//! - `leptos_components/small_components/app_border.rs` — tracked.
//! - `leptos_components/main_content/connection_indicator.rs` — tracked.
//! - `leptos_components/reconcile_page/reconcile_page.rs` — tracked.
//! - `expose_methods_for_testing/method_read_conn_state.rs` — untracked.
//! - `db/dev_actions.rs` — untracked (dev-panel actions run outside
//!   any reactive scope).
//!
//! Setters (literal path from the web_interface `src/` root). Each
//! entry says what would stop happening if that setter didn't exist.
//! - `owner_of_interacting_with_server/core.rs` — `TwoTabs`: the
//!   two-tab guard couldn't stop the second tab from talking to the
//!   server. `Connecting`: the UI would show the previous state while a
//!   socket opens. `LoggingIn`: sends could fire before login and the
//!   drift check finish. `Offline` (socket failed): a dead socket would
//!   keep looking live. `Offline` (browser offline event): losing the
//!   network wouldn't stop sync, and the app would keep trying to send.
//! - `owner_of_interacting_with_server/main_for_login_and_registration.rs`
//!   — `Connected`: sync would never unblock, so nothing would ever be
//!   sent or received.
//! - `owner_of_interacting_with_server/main_for_receiving_spontaneous_love_letters_from_server.rs`
//!   — `FailedToConnect`: a fatal receive error would leave the client
//!   claiming it's connected.
//! - `owner_of_interacting_with_server/drift/enter_lonewolf.rs` —
//!   `Lonewolf` / `LoneWolfAndOffline`: sync wouldn't stop on drift, so
//!   edits built on stale data would keep flowing both ways.

use std::sync::LazyLock;

use leptos::prelude::*;

pub use drift_guard_client::connection_status::ConnectionStatus as ConnState;

static CONN_STATE: LazyLock<ArcRwSignal<ConnState>> =
    LazyLock::new(|| ArcRwSignal::new(ConnState::Offline));

/// Current state. Subscribes.
pub fn plexget_conn_state() -> ConnState {
    CONN_STATE.get()
}

/// Current state without subscribing. For non-reactive code that just
/// needs the value once.
pub fn plexget_conn_state_untracked() -> ConnState {
    CONN_STATE.get_untracked()
}

/// Write the current state.
pub fn plexset_conn_state(state: ConnState) {
    CONN_STATE.set(state);
}