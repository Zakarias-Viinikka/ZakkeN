//! Android socket half.
//!
//! Same contract as `socket_web`: one connect attempt per signal from
//! `rx_attempt_to_connect`, drain the outgoing queue to the wire, decode
//! incoming frames into `Response` and hand them to the router's
//! dispatch loop. Retry is driven from outside, not here.
//!
//! Kotlin has no tokio runtime, so this file owns one (`runtime()`),
//! built lazily on first use and kept for the life of the process.

use std::sync::OnceLock;
use std::time::Duration;

use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};
use futures::{SinkExt, StreamExt};
use server_protocol::{json_str_to_response, message_to_json_str, Message, Response};
use tokio::net::TcpStream;
use tokio::runtime::{Builder, Runtime};
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

use crate::conn_status::ConnStatus;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

fn runtime() -> Result<&'static Runtime, String> {
    if let Some(rt) = RUNTIME.get() {
        return Ok(rt);
    }
    let rt = Builder::new_multi_thread()
        .worker_threads(2)
        .thread_name("zakken-socket")
        .enable_all()
        .build()
        .map_err(|e| format!("build tokio runtime: {e}"))?;
    // If another thread won the race its runtime is the one we use; ours
    // is dropped here, which is fine because it never ran anything.
    let _ = RUNTIME.set(rt);
    RUNTIME
        .get()
        .ok_or_else(|| "runtime missing right after set".to_string())
}

pub(crate) fn spawn<S, F>(
    url: String,
    outgoing_rx: UnboundedReceiver<Message>,
    incoming_tx: UnboundedSender<Response>,
    mut on_conn_status: S,
    on_disconnect: F,
    rx_attempt_to_connect: UnboundedReceiver<()>,
) where
    S: FnMut(ConnStatus) + Send + 'static,
    F: Fn() + Send + 'static,
{
    let rt = match runtime() {
        Ok(rt) => rt,
        Err(e) => {
            log::error!("socket_android::spawn: {e}");
            on_conn_status(ConnStatus::Failed);
            return;
        }
    };
    rt.spawn(async move {
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

pub(crate) fn spawn_dispatch(router: crate::router::Router) {
    let rt = match runtime() {
        Ok(rt) => rt,
        Err(e) => {
            log::error!("socket_android::spawn_dispatch: {e}");
            return;
        }
    };
    rt.spawn(async move {
        router.run().await;
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
    S: FnMut(ConnStatus) + Send + 'static,
    F: Fn() + Send + 'static,
{
    loop {
        let Some(()) = rx_attempt_to_connect.next().await else {
            return;
        };
        // One connect attempt per burst of signals.
        while rx_attempt_to_connect.try_recv().is_ok() {}

        on_conn_status(ConnStatus::Connecting);
        match open(&url).await {
            Ok(ws) => {
                on_conn_status(ConnStatus::Connected);
                let keep_going = run_one(ws, &mut outgoing_rx, incoming_tx.clone()).await;
                on_disconnect();
                if !keep_going {
                    return;
                }
                on_conn_status(ConnStatus::Failed);
            }
            Err(e) => {
                log::warn!("connect {url}: {e}");
                on_conn_status(ConnStatus::Failed);
            }
        }
    }
}

async fn open(url: &str) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, String> {
    match tokio::time::timeout(CONNECT_TIMEOUT, connect_async(url)).await {
        Ok(Ok((ws, _response))) => Ok(ws),
        Ok(Err(e)) => Err(e.to_string()),
        Err(_) => Err(format!("timed out after {}s", CONNECT_TIMEOUT.as_secs())),
    }
}

/// One connection. Runs until the socket closes. Returns true if the
/// socket died (caller should retry later), false if the outgoing
/// channel closed (nothing left to send, stop entirely).
async fn run_one(
    ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
    outgoing_rx: &mut UnboundedReceiver<Message>,
    incoming_tx: UnboundedSender<Response>,
) -> bool {
    let (mut write, mut read) = ws.split();
    loop {
        tokio::select! {
            msg = outgoing_rx.next() => {
                match msg {
                    Some(m) => {
                        let json = match message_to_json_str(&m) {
                            Ok(j) => j,
                            Err(e) => {
                                log::error!("serialize Message: {e:?}");
                                continue;
                            }
                        };
                        if let Err(e) = write.send(WsMessage::Text(json.into())).await {
                            log::warn!("socket send: {e}");
                            return true;
                        }
                    }
                    None => return false,
                }
            }
            frame = read.next() => {
                match frame {
                    Some(Ok(WsMessage::Text(text))) => {
                        match json_str_to_response(text.as_str()) {
                            Ok(r) => {
                                if incoming_tx.unbounded_send(r).is_err() {
                                    return false;
                                }
                            }
                            Err(e) => log::error!("decode Response: {e:?}"),
                        }
                    }
                    Some(Ok(WsMessage::Close(_))) | None => return true,
                    // Ping/pong are answered by tungstenite; binary isn't part of the protocol.
                    Some(Ok(_)) => {}
                    Some(Err(e)) => {
                        log::warn!("socket read: {e}");
                        return true;
                    }
                }
            }
        }
    }
}