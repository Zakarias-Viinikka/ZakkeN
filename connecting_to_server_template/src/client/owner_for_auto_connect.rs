//! Owner for "connect to the server the first time the user presses
//! Start client".
//!
//! Watches `client_started_signal`. On the first flip to true, sets a
//! local `auto_connect_seen` flag so it doesn't run twice, then calls
//! `connect_to_server_and_announce` (which goes through `restart_sync_stack`
//! and pops the outcome).
//!
//! Caller pov: call `spawn_owner_for_auto_connect(client_handle)` once,
//! in `AppMain`. Done.

use crate::client_server_coms::ComsHandleForClient;
use leptos::prelude::*;

pub fn main_for_connecting_the_client_once_when_it_starts(client_handle: ComsHandleForClient) {
    let auto_connect_seen = RwSignal::new(false);
    Effect::new(move |_| {
        if !crate::client::client_started_signal().get() {
            return;
        }
        if auto_connect_seen.get_untracked() {
            return;
        }
        auto_connect_seen.set(true);
        crate::ui_ops::connect_to_server_and_announce(client_handle.clone());
    });
}
