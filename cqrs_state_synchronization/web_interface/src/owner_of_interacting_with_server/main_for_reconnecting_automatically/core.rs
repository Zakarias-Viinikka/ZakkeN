//! Caller pov: `spawn_task_to_auto_reconnect()` starts a loop that
//! runs once per second. Each tick reads `ConnState` and, if the
//! client is offline (or offline-and-gated), asks the router to try
//! opening a connection. If a connection attempt is already in flight
//! or the client is connected or gated, the tick does nothing.
//!
//! The socket task coalesces extra attempts, so calling
//! `attempt_reconnect` more often than strictly necessary is fine.

use gloo_timers::future::TimeoutFuture;
use leptos::task::spawn_local;

use crate::complex_state::router_handle;
use crate::plexget_conn_state_untracked;
use crate::owner_of_interacting_with_server::ConnState;

pub fn spawn_task_to_auto_reconnect() {
    spawn_local(async move {
        loop {
            if should_i_reconnect() {
                let _ = router_handle::attempt_reconnect();
            }
            TimeoutFuture::new(1000).await;
        }
    });
}

/// True only for the states where the client should be trying to
/// reconnect. False for `Connecting`, `LoggingIn`, `Connected`,
/// `Lonewolf`, `TwoTabs` (already trying, or connected, or blocked by
/// something other than the network). When `OfflineDeliberately` lands,
/// add it here as false.
fn should_i_reconnect() -> bool {
    matches!(
        plexget_conn_state_untracked(),
        ConnState::Offline | ConnState::FailedToConnect | ConnState::LonewolfAndOffline
    )
}