use std::sync::LazyLock;
use std::sync::atomic::{AtomicBool, Ordering};

use leptos::prelude::*;
use leptos::task::spawn_local;
use shared_internals::{ConnStatus, Router};
use wasm_bindgen::prelude::*;

use crate::db::batcher::BatcherHandle;
use crate::db::cached_ids;
use crate::page_state::OpenPages;

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

static LINK_STARTED: AtomicBool = AtomicBool::new(false);

/// Connect to the server. Called from the auto-connect effect in
/// `main.rs` and from the exposed `connect_to_socket` method. Idempotent:
/// subsequent calls after the first successful start do nothing.
pub fn connect_to_socket(batcher: BatcherHandle, open: OpenPages) {
    let init_done = cached_ids::INIT_DONE.get_untracked();
    let session = cached_ids::SESSION_ID.get_untracked();
    let user = cached_ids::USER_ID.get_untracked();
    if !init_done || session.is_none() || user.is_none() {
        return;
    }
    if super::server_port::get_port().is_none() && not_on_the_custom_port_page() {
        super::server_port::set_port(3000);
    }
    if super::connecting_gate::CONNECTING_GATE.get_untracked()
        != super::connecting_gate::ConnectingGate::AllowedToConnect
    {
        return;
    }
    if LINK_STARTED.swap(true, Ordering::SeqCst) {
        return;
    }
    spawn_local(async move {
        start_link(batcher, open).await;
    });
}

async fn start_link(batcher: BatcherHandle, open: OpenPages) {
    if !super::two_tab_guard::acquire().await {
        CONN_STATE.set(ConnState::TwoTabs);
        return;
    }

    install_check_if_internet_connection_is_lost_and_set_conn_state_to_offline();

    let url = build_ws_url();
    let (
        tx_give_love_letter_that_came_from_server_to_task_that_processes_it,
        rx_take_love_letter_from_server_and_process_it,
    ) = super::make_letters_channel();

    // rxtx for task_that_receives_socket_messages
    // talks to the thing that owns receiving
    // spontaneous love letters
    let (
        tx_tell_receiving_task_that_socket_reconnected_so_it_can_relogin,
        rx_hear_that_socket_reconnected_so_we_can_relogin,
    ) = futures_channel::mpsc::unbounded::<()>();

    let on_love_letter_from_server = move |letter| {
        let _ = tx_give_love_letter_that_came_from_server_to_task_that_processes_it
            .unbounded_send(letter);
    };
    let on_conn_status_change = move |status| match status {
        ConnStatus::Connecting => CONN_STATE.set(ConnState::Connecting),
        ConnStatus::Connected => {
            CONN_STATE.set(ConnState::Connected);
            let _ =
                tx_tell_receiving_task_that_socket_reconnected_so_it_can_relogin.unbounded_send(());
        }
        ConnStatus::Failed => CONN_STATE.set(ConnState::FailedToConnect),
    };

    let router = Router::new(url, on_love_letter_from_server, on_conn_status_change);

    crate::shared_owners::router_handle::set(router.clone());

    super::spawn_sender(router.clone());
    super::spawn_task_for_receiving_socket_messages(
        router,
        rx_take_love_letter_from_server_and_process_it,
        rx_hear_that_socket_reconnected_so_we_can_relogin,
        batcher,
        open,
    );
}

fn not_on_the_custom_port_page() -> bool {
    let Some(window) = web_sys::window() else {
        return true;
    };
    let Ok(pathname) = window.location().pathname() else {
        return true;
    };
    pathname != "/custom_port_setting_page"
}

fn build_ws_url() -> String {
    let port = super::server_port::get_port()
        .expect("failing to get port by now is a logic error");
    let Some(window) = web_sys::window() else {
        return format!("ws://127.0.0.1:{port}/ws");
    };
    let Ok(loc) = js_sys::Reflect::get(window.as_ref(), &JsValue::from_str("location")) else {
        return format!("ws://127.0.0.1:{port}/ws");
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
    format!("{scheme}://{hostname}:{port}/ws")
}

fn install_check_if_internet_connection_is_lost_and_set_conn_state_to_offline() {
    let Some(window) = web_sys::window() else {
        return;
    };
    let on_offline = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
        CONN_STATE.set(ConnState::Offline);
    });
    let _ = window.add_event_listener_with_callback("offline", on_offline.as_ref().unchecked_ref());
    on_offline.forget();
}