//! The "server is on" flag.
//!
//! Template-specific shared owner. Only exists because the template's
//! transport is an mpsc channel that never dies — turning the server
//! "off" has to be a flag that refuses to reply, so a real-world
//! "server went dark" can be simulated without killing the channel. A
//! real client/server setup has no such flag: the client knows the
//! server is reachable because the socket is up, and it tracks that
//! itself (ConnState). If this gets ported, delete this file and let
//! the socket state do the job.
//!
//! Caller pov: `powered_signal()` returns the signal. Read it, don't
//! write it.
//!
//! Internals pov: writers are the `TurnOn` / `TurnOff` handlers in
//! `server/`. Readers are `server/` (border colour, the `AreYouOnline`
//! reply, and the reply-drop gate in `latency.rs`),
//! `client/components/controls.rs` (button enable state), and
//! `server/panel.rs` (the readout).

use leptos::prelude::*;
use std::sync::LazyLock;

pub(crate) static POWERED: LazyLock<ArcRwSignal<bool>> =
    LazyLock::new(|| ArcRwSignal::new(false));

pub fn powered_signal() -> ArcRwSignal<bool> {
    POWERED.clone()
}