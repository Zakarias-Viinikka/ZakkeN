//! Start the link to zakken_server.
//!
//! `Router::new(url, on_letters, on_status)` opens the socket and runs
//! the retry loop internally. This module decides when to start, holds
//! the two-tab lock, and hands the resulting `RouterHandle` to the
//! worker and the sender.

use std::sync::{Arc, LazyLock, Mutex};

use leptos::prelude::*;
use leptos::task::spawn_local;
use shared_internals_for_server::{ConnStatus, Router};
use wasm_bindgen::prelude::*;

use crate::complicated_signal_flow::{BatcherHandle, OpenPages};
use crate::db::cached_ids;

/// Where the link is right now. Drives the MainContent border.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnState {
    Offline,
    Connecting,
    FailedToConnect,
    Connected,
    TwoTabs,
}

pub static CONN_STATE: LazyLock<ArcRwSignal<ConnState>> =
    LazyLock::new(|| ArcRwSignal::new(ConnState::Offline));

static STARTED: LazyLock<Mutex<bool>> = LazyLock::new(|| Mutex::new(false));

pub fn start_server_link(batcher: BatcherHandle, open: OpenPages) {
    Effect::new(move |_| {
        if *STARTED.lock().unwrap() {
            return;
        }
        if !cached_ids::INIT_DONE.get() {
            return;
        }
        if cached_ids::SESSION_ID.get().is_none() {
            return;
        }
        if cached_ids::USER_ID.get().is_none() {
            return;
        }
        *STARTED.lock().unwrap() = true;
        let batcher_clone = batcher.clone();
        spawn_local(async move {
            start_link(batcher_clone, open).await;
        });
    });
}

async fn start_link(batcher: BatcherHandle, open: OpenPages) {
    if !super::two_tab::acquire().await {
        CONN_STATE.set(ConnState::TwoTabs);
        return;
    }

    install_offline_listener();

    let url = build_ws_url();
    let (letters_tx, letters_rx) = super::make_letters_channel();

    // Fires on every RE-connect (not the first connect). Each transport
    // blip makes a fresh server-side socket with no user attached, so
    // run_worker listens here and re-runs login to re-authenticate.
    let (reconnect_tx, reconnect_rx) = futures_channel::mpsc::unbounded::<()>();
    let first_connected = Arc::new(std::sync::atomic::AtomicBool::new(true));

    let first_connected_cb = Arc::clone(&first_connected);
    let router = Router::new(
        url,
        move |letter| {
            let _ = letters_tx.unbounded_send(letter);
        },
        move |status| match status {
            ConnStatus::Connecting => CONN_STATE.set(ConnState::Connecting),
            ConnStatus::Connected => {
                CONN_STATE.set(ConnState::Connected);
                if !first_connected_cb.swap(false, std::sync::atomic::Ordering::Relaxed) {
                    let _ = reconnect_tx.unbounded_send(());
                }
            }
            ConnStatus::Failed => CONN_STATE.set(ConnState::FailedToConnect),
        },
    );

    super::spawn_sender(router.clone());
    super::spawn_worker(router, letters_rx, reconnect_rx, batcher, open);
}

fn build_ws_url() -> String {
    let Some(window) = web_sys::window() else {
        return "ws://127.0.0.1:3000/ws".to_string();
    };
    let Ok(loc) = js_sys::Reflect::get(window.as_ref(), &JsValue::from_str("location")) else {
        return "ws://127.0.0.1:3000/ws".to_string();
    };
    let protocol = js_sys::Reflect::get(&loc, &JsValue::from_str("protocol"))
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_default();
    let hostname = js_sys::Reflect::get(&loc, &JsValue::from_str("hostname"))
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_else(|| "127.0.0.1".to_string());
    let scheme = if protocol == "https:" { "wss" } else { "ws" };
    format!("{scheme}://{hostname}:3000/ws")
}

/// When the browser reports offline, show it. The router keeps retrying
/// in the background; its next status flip will clear the Offline state.
fn install_offline_listener() {
    let Some(window) = web_sys::window() else {
        return;
    };
    let on_offline = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
        CONN_STATE.set(ConnState::Offline);
    });
    let _ = window
        .add_event_listener_with_callback("offline", on_offline.as_ref().unchecked_ref());
    on_offline.forget();
}