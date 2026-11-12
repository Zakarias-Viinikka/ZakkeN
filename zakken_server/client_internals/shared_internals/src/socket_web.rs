//! Browser socket half.
//!
//! Opens a WebSocket when told to, drains the outgoing queue to the
//! wire, decodes incoming frames into `Response`, and feeds them back
//! into the router's dispatch loop. One connect attempt per signal
//! from `rx_attempt_to_connect`. On close or open failure, reports
//! through `on_conn_status` and waits for the next signal.
//!
//! Retry is not this file's job any more — it's driven from outside by
//! `web_interface::owner_of_interacting_with_server::main_for_reconnecting_automatically`.
//!
//! The outgoing queue is owned by this task for the lifetime of the
//! process. A dropped receiver would lose every queued message, so it
//! lives here.

use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::StreamExt;
use gloo_timers::future::TimeoutFuture;
use server_protocol::{json_str_to_response, message_to_json_str, Message, Response};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{MessageEvent, WebSocket};

use crate::conn_status::ConnStatus;

pub(crate) fn spawn<S, F>(
    url: String,
    outgoing_rx: UnboundedReceiver<Message>,
    incoming_tx: UnboundedSender<Response>,
    on_conn_status: S,
    on_disconnect: F,
    rx_attempt_to_connect: UnboundedReceiver<()>,
) where
    S: FnMut(ConnStatus) + 'static,
    F: Fn() + 'static,
{
    spawn_local(async move {
        socket_task(
            url,
            outgoing_rx,
            incoming_tx,
            on_conn_status,
            on_disconnect,
            rx_attempt_to_connect,
        )
        .await;
    });
}

async fn socket_task<S, F>(
    url: String,
    mut outgoing_rx: UnboundedReceiver<Message>,
    incoming_tx: UnboundedSender<Response>,
    mut on_conn_status: S,
    on_disconnect: F,
    mut rx_attempt_to_connect: UnboundedReceiver<()>,
) where
    S: FnMut(ConnStatus) + 'static,
    F: Fn() + 'static,
{
    loop {
        // Wait for the app to ask for an attempt.
        let Some(()) = rx_attempt_to_connect.next().await else {
            return;
        };
        // Drain any extra signals so one connect attempt doesn't trigger
        // several back-to-back open attempts.
        while rx_attempt_to_connect.try_recv().is_ok() {}

        on_conn_status(ConnStatus::Connecting);
        match open_and_wait(&url).await {
            Ok(ws) => {
                on_conn_status(ConnStatus::Connected);
                let keep_going = run_one(ws, &mut outgoing_rx, incoming_tx.clone()).await;
                on_disconnect();
                if !keep_going {
                    return;
                }
                // Signal "we're no longer connected, the retry task
                // should try again". Failed maps to Offline in the app.
                on_conn_status(ConnStatus::Failed);
            }
            Err(e) => {
                web_sys::console::warn_1(&format!("connect {url}: {e}").into());
                on_conn_status(ConnStatus::Failed);
            }
        }
    }
}

async fn open_and_wait(url: &str) -> Result<WebSocket, String> {
    let ws = WebSocket::new(url).map_err(|e| format!("{e:?}"))?;
    loop {
        match ws.ready_state() {
            WebSocket::OPEN => return Ok(ws),
            WebSocket::CLOSING | WebSocket::CLOSED => {
                return Err("closed during open".to_string());
            }
            _ => TimeoutFuture::new(20).await,
        }
    }
}

/// One connection. Runs until the socket closes. Returns true if the
/// caller should keep the process alive (socket died), false if the
/// outgoing channel closed (nothing left to send, stop entirely).
async fn run_one(
    ws: WebSocket,
    outgoing_rx: &mut UnboundedReceiver<Message>,
    incoming_tx: UnboundedSender<Response>,
) -> bool {
    let (resp_tx, mut resp_rx) = unbounded::<Response>();
    let (close_tx, mut close_rx) = unbounded::<()>();

    let onmessage = {
        let tx = resp_tx.clone();
        Closure::<dyn FnMut(MessageEvent)>::new(move |e: MessageEvent| {
            let Some(text) = e.data().as_string() else {
                return;
            };
            match json_str_to_response(&text) {
                Ok(r) => {
                    let _ = tx.unbounded_send(r);
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("decode Response: {e:?}").into());
                }
            }
        })
    };
    ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));

    let onclose = {
        let tx = close_tx.clone();
        Closure::<dyn FnMut(JsValue)>::new(move |_| {
            let _ = tx.unbounded_send(());
        })
    };
    ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));

    let onerror = {
        let tx = close_tx.clone();
        Closure::<dyn FnMut(JsValue)>::new(move |_| {
            let _ = tx.unbounded_send(());
        })
    };
    ws.set_onerror(Some(onerror.as_ref().unchecked_ref()));

    let _onmessage = onmessage;
    let _onclose = onclose;
    let _onerror = onerror;

    loop {
        futures::select! {
            msg = outgoing_rx.next() => {
                match msg {
                    Some(m) => {
                        let json = match message_to_json_str(&m) {
                            Ok(j) => j,
                            Err(e) => {
                                web_sys::console::error_1(
                                    &format!("serialize Message: {e:?}").into(),
                                );
                                continue;
                            }
                        };
                        if ws.send_with_str(&json).is_err() {
                            return true;
                        }
                    }
                    None => return false,
                }
            }
            resp = resp_rx.next() => {
                match resp {
                    Some(r) => {
                        if incoming_tx.unbounded_send(r).is_err() {
                            return false;
                        }
                    }
                    None => return true,
                }
            }
            _ = close_rx.next() => {
                return true;
            }
        }
    }
}

pub(crate) fn spawn_dispatch(router: crate::router::Router) {
    wasm_bindgen_futures::spawn_local(async move {
        router.run().await;
    });
}