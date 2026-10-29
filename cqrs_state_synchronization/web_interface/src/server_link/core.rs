use std::sync::LazyLock;

use leptos::prelude::*;
use leptos::task::spawn_local;
use shared_internals_for_server::{ConnStatus, Router};
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

pub fn start_server_link(batcher: BatcherHandle, open: OpenPages) {
    Effect::new(move |_| {
        if !cached_ids::INIT_DONE.get() {
            return;
        }
        if cached_ids::SESSION_ID.get_untracked().is_none() {
            return;
        }
        if cached_ids::USER_ID.get_untracked().is_none() {
            return;
        }
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

    super::spawn_sender(router.clone());
    super::spawn_task_for_receiving_socket_messages(
        router,
        rx_take_love_letter_from_server_and_process_it,
        rx_hear_that_socket_reconnected_so_we_can_relogin,
        batcher,
        open,
    );
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
