//! Owner for "turn the server on when the user presses Start server".
//!
//! Watches the `server_started` signal from `AppMain`. The first time
//! it flips true, sends `TurnOn` to the fake server. Reactive, not
//! one-shot — but the signal only flips once, so in practice this
//! fires once.
//!
//! Caller pov: call `spawn_owner_for_auto_power_on(server_started,
//! client_handle)` once, in `AppMain`. Done.

use crate::client_server_coms::ComsHandleForClient;
use leptos::prelude::*;

pub fn main_for_powering_the_server_when_the_user_starts_it(
    server_started: ReadSignal<bool>,
    client_handle: ComsHandleForClient,
) {
    Effect::new(move |_| {
        if server_started.get() {
            client_handle.turn_on_server();
        }
    });
}
