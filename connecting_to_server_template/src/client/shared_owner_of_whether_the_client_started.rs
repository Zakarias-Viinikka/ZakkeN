//! The "the user pressed Start client" flag.
//!
//! Lives in `client/` because the client owns it — the flip to true is
//! driven by `AppMain`'s `<ClientFallback on_start=.../>`. Everyone
//! else just reads.
//!
//! Caller pov: `client_started_signal()` returns the signal. Read it,
//! don't write it. Only `AppMain` writes (through `client_started_signal().set(true)`).
//!
//! Internals pov: writers are the `ClientFallback` on_start callback in
//! `app_main.rs`. Readers are `client/owner_for_auto_connect.rs`
//! (auto-connect once), `client/components/controls.rs` (button enable
//! state), and `app_main.rs` (the `<Show>` that swaps fallback for the
//! real panel).

use leptos::prelude::*;
use std::sync::LazyLock;

pub fn client_started_signal() -> ArcRwSignal<bool> {
    CLIENT_STARTED.clone()
}

static CLIENT_STARTED: LazyLock<ArcRwSignal<bool>> =
    LazyLock::new(|| ArcRwSignal::new(false));
