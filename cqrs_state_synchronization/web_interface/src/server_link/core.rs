//! The link between the web app and zakken_server.
//!
//! Step 1: connect, register/login, and drive the connection-state
//! signal that the border in MainContent reads. Later steps add
//! bootstrap, the receive worker, UI refresh, and sending.

use std::sync::{LazyLock, Mutex};

use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsValue;

use client_web::ConnEvent;

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

/// Only one link per tab. Set on the first fire of `start_server_link`'s
/// effect where INIT_DONE, SESSION_ID, and USER_ID are all ready.
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
            run_link_loop(batcher_clone, open).await;
        });
    });
}

enum ConnOutcome {
    Closed,
    Failed,
}

async fn run_link_loop(batcher: BatcherHandle, open: OpenPages) {
    if !super::two_tab::acquire().await {
        CONN_STATE.set(ConnState::TwoTabs);
        return;
    }
    let mut backoff_ms: u32 = 1000;
    loop {
        if !is_online() {
            CONN_STATE.set(ConnState::Offline);
            wait_for_online().await;
            backoff_ms = 1000;
            continue;
        }
        CONN_STATE.set(ConnState::Connecting);
        let outcome = run_one_connection(batcher.clone(), open).await;
        match outcome {
            ConnOutcome::Closed => backoff_ms = 1000,
            ConnOutcome::Failed => CONN_STATE.set(ConnState::FailedToConnect),
        }
        TimeoutFuture::new(backoff_ms).await;
        backoff_ms = backoff_ms.saturating_mul(2).min(30_000);
    }
}

async fn run_one_connection(batcher: BatcherHandle, open: OpenPages) -> ConnOutcome {
    let url = build_ws_url();
    let (letters_tx, letters_rx) = super::make_letters_channel();
    let mut client = match client_web::connect(&url, move |batch| {
        let _ = letters_tx.unbounded_send(batch);
    }) {
        Ok(c) => c,
        Err(e) => {
            leptos::logging::error!("connect {url}: {e}");
            return ConnOutcome::Failed;
        }
    };

    // Login lives inside the receive worker now, so bootstrap and the
    // caught-up pull can't run before the user is identified. The
    // sender starts in parallel; it only fires on kick, and the worker
    // kicks after a successful login.
    let router = client.router_clone();
    super::spawn_sender(router.clone());
    super::spawn_worker(router.clone(), letters_rx, batcher, open);

    let mut was_connected = false;
    while let Some(ev) = client.next_event().await {
        match ev {
            ConnEvent::Opened => {
                CONN_STATE.set(ConnState::Connected);
                was_connected = true;
            }
            ConnEvent::Closed(_) => {
                CONN_STATE.set(ConnState::Connecting);
                client.close();
                return if was_connected {
                    ConnOutcome::Closed
                } else {
                    ConnOutcome::Failed
                };
            }
            ConnEvent::Errored(_) => {
                client.close();
                return ConnOutcome::Failed;
            }
        }
    }
    client.close();
    if was_connected {
        ConnOutcome::Closed
    } else {
        ConnOutcome::Failed
    }
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

fn is_online() -> bool {
    let Some(window) = web_sys::window() else {
        return true;
    };
    let navigator = window.navigator();
    let Ok(val) = js_sys::Reflect::get(navigator.as_ref(), &JsValue::from_str("onLine")) else {
        return true;
    };
    val.as_bool().unwrap_or(true)
}

async fn wait_for_online() {
    loop {
        TimeoutFuture::new(1000).await;
        if is_online() {
            return;
        }
    }
}
