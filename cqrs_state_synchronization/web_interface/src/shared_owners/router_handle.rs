use std::sync::OnceLock;

use shared_internals::{build_request, ClientCommand, RouterHandle};

/// Holds the app's single `RouterHandle`.
///
/// `start_link` calls `set` once the socket exists. Callers use `send` and
/// never see the handle, the static, or how it's stored.
///
/// What a command becomes on the wire is decided in
/// `shared_internals::request_builder`. This file only sends.
static ROUTER_HANDLE: OnceLock<RouterHandle> = OnceLock::new();

pub fn set(router_handle: RouterHandle) {
    if ROUTER_HANDLE.set(router_handle).is_err() {
        panic!("router handle was set twice");
    }
}

pub async fn send(command: ClientCommand) -> Result<Vec<u8>, String> {
    let handle = ROUTER_HANDLE
        .get()
        .ok_or_else(|| "not connected to the server".to_string())?;
    let (request, payload) = build_request(command);
    handle
        .send_request(request, payload)
        .await
        .map_err(|e| format!("{e:?}"))
}