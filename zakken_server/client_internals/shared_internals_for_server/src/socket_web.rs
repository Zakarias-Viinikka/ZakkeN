//! Browser socket half.
//!
//! Opens a WebSocket, drains the outgoing queue to the wire, decodes
//! incoming frames into `Response` and feeds them back into the
//! router's dispatch loop. Owns the retry loop: on every failed open,
//! or on close after a successful connect, reports to `on_conn_status`
//! and retries with backoff.
//!
//! The outgoing queue is owned by the retry loop and lent to each
//! connect attempt. A dropped receiver would lose every queued
//! message, so it lives here for the lifetime of the process.

use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::StreamExt;
use gloo_timers::future::TimeoutFuture;
use server_protocol::{json_str_to_response, message_to_json_str, Message, Response};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{MessageEvent, WebSocket};

use crate::conn_status::ConnStatus;

pub(crate) fn spawn<S>(
    url: String,
    outgoing_rx: UnboundedReceiver<Message>,
    incoming_tx: UnboundedSender<Response>,
    on_conn_status: S,
) where
    S: FnMut(ConnStatus) + 'static,
{
    spawn_local(async move {
        run_retry(url, outgoing_rx, incoming_tx, on_conn_status).await;
    });
}

async fn run_retry<S>(
    url: String,
    mut outgoing_rx: UnboundedReceiver<Message>,
    incoming_tx: UnboundedSender<Response>,
    mut on_conn_status: S,
) where
    S: FnMut(ConnStatus) + 'static,
{
    let mut backoff_ms: u32 = 1000;
    loop {
        on_conn_status(ConnStatus::Connecting);
        match open_and_wait(&url).await {
            Ok(ws) => {
                on_conn_status(ConnStatus::Connected);
                let closed_cleanly = run_one(ws, &mut outgoing_rx, incoming_tx.clone()).await;
                if !closed_cleanly {
                    // The message channel itself closed; nothing more
                    // to do.
                    return;
                }
                backoff_ms = 1000;
                on_conn_status(ConnStatus::Connecting);
            }
            Err(e) => {
                web_sys::console::warn_1(&format!("connect {url}: {e}").into());
                on_conn_status(ConnStatus::Failed);
            }
        }
        TimeoutFuture::new(backoff_ms).await;
        backoff_ms = backoff_ms.saturating_mul(2).min(30_000);
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
/// caller should retry (socket died), false if the outgoing channel
/// closed (nothing left to send, stop entirely).
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

    // Hold the closures for the duration of `run_one`.
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
