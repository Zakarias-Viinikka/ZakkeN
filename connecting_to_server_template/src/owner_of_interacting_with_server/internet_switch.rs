//! Client-side "my internet went down" simulation. The template's
//! channel is always up, so without this the offline retry loop would
//! always see the server as reachable. While off: is_server_online
//! reports false, incoming messages are dropped, and turning back on
//! lets the retry loop reconnect normally.

use leptos::prelude::*;
use std::sync::LazyLock;

static INTERNET_ON: LazyLock<ArcRwSignal<bool>> =
    LazyLock::new(|| ArcRwSignal::new(true));

pub fn internet_on_signal() -> ArcRwSignal<bool> {
    INTERNET_ON.clone()
}

pub fn is_internet_on() -> bool {
    INTERNET_ON.get_untracked()
}

pub fn turn_internet_off() {
    if !is_internet_on() {
        return;
    }
    INTERNET_ON.set(false);
    crate::owner_of_interacting_with_server::conn_state::go_offline_after_connection_lost();
}

pub fn turn_internet_on() {
    INTERNET_ON.set(true);
}
