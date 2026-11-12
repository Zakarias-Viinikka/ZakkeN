use std::sync::OnceLock;

use shared_internals::{build_request, ClientCommand, RouterHandle};

/// Holds the app's single `RouterHandle`.
///
/// `start_link` calls `set` once the socket exists. Callers use `send` and
/// never see the handle, the static, or how it's stored.
///
/// What a command becomes on the wire is decided in
/// `shared_internals::request_builder`. This file only sends.
///
/// `send` returns the raw `ServErr` so callers can match on variants
/// like `DriftVersionMismatch`. Callers that just need a message can
/// `format!("{e:?}")` it.
static ROUTER_HANDLE: OnceLock<RouterHandle> = OnceLock::new();

pub fn set(router_handle: RouterHandle) {
    if ROUTER_HANDLE.set(router_handle).is_err() {
        panic!("router handle was set twice");
    }
}

pub async fn send(command: ClientCommand) -> Result<Vec<u8>, server_protocol::ServErr> {
    let handle = ROUTER_HANDLE.get().ok_or_else(|| {
        server_protocol::ServErr::BadCode(
            "complex_state::router_handle::send: router handle not installed".to_string(),
        )
    })?;
    let (request, payload) = build_request(command);
    handle.send_request(request, payload).await
}

/// Ask the socket task to try opening a new connection. Returns an
/// error only if the router hasn't been installed yet. Safe to call
/// repeatedly; extra attempts while one is in flight are coalesced
/// inside the socket task.
pub fn attempt_reconnect() -> Result<(), server_protocol::ServErr> {
    let handle = ROUTER_HANDLE.get().ok_or_else(|| {
        server_protocol::ServErr::BadCode(
            "complex_state::router_handle::attempt_reconnect: router handle not installed"
                .to_string(),
        )
    })?;
    handle.attempt_reconnect();
    Ok(())
}