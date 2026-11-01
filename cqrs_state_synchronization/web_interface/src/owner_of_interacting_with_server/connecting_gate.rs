use std::sync::LazyLock;

use leptos::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectingGate {
    Blocked,
    AllowedToConnect,
}

pub static CONNECTING_GATE: LazyLock<ArcRwSignal<ConnectingGate>> = LazyLock::new(|| {
    let initial = if super::server_port::get_port().is_some() {
        ConnectingGate::AllowedToConnect
    } else {
        ConnectingGate::Blocked
    };
    ArcRwSignal::new(initial)
});

pub fn allow_connecting() {
    CONNECTING_GATE.set(ConnectingGate::AllowedToConnect);
}