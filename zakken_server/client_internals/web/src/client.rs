//! The browser half of talking to zakken_server.
//!
//! Opens a WebSocket, writes Messages from the router's outgoing channel,
//! decodes incoming frames into Responses and forwards them to the router.
//! Holds everything in a ClientHandle; dropping it closes the socket and
//! cancels the pending waits.

use futures::channel::mpsc::{unbounded, UnboundedReceiver};
use futures::StreamExt;
use server_protocol::{UnsentLetter, json_str_to_response, message_to_json_str};
use shared_internals_for_server::{ResponseRouter, RouterHandles};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{MessageEvent, WebSocket};

pub enum ConnEvent {
    Opened,
    Closed(String),
    Errored(String),
}

pub struct ClientHandle {
    router: ResponseRouter,
    conn_events: UnboundedReceiver<ConnEvent>,
    ws: WebSocket,
    _onmessage: Option<Closure<dyn FnMut(MessageEvent)>>,
    _onopen: Option<Closure<dyn FnMut(JsValue)>>,
    _onclose: Option<Closure<dyn FnMut(JsValue)>>,
    _onerror: Option<Closure<dyn FnMut(JsValue)>>,
}

impl ClientHandle {
    pub fn router_clone(&self) -> ResponseRouter {
        self.router.clone()
    }

    pub async fn next_event(&mut self) -> Option<ConnEvent> {
        self.conn_events.next().await
    }

    /// Drop the connection: fail every pending request, unregister the JS
    /// handlers (so their closures can be dropped and their senders
    /// released), and close the socket.
    pub fn close(mut self) {
        self.router.fail_all_pending();
        self.ws.set_onmessage(None);
        self.ws.set_onopen(None);
        self.ws.set_onclose(None);
        self.ws.set_onerror(None);
        let _ = self.ws.close();
        self._onmessage = None;
        self._onopen = None;
        self._onclose = None;
        self._onerror = None;
    }
}

pub fn connect<F>(url: &str, on_love_letters: F) -> Result<ClientHandle, String>
where
    F: FnMut(Vec<UnsentLetter>) + 'static,
{
    let ws = WebSocket::new(url).map_err(|e| format!("WebSocket::new: {e:?}"))?;

    let (router, handles, routing) = ResponseRouter::new(on_love_letters);
    spawn_local(routing);

    let RouterHandles {
        outgoing,
        incoming_responses_tx,
    } = handles;

    let (conn_tx, conn_rx) = unbounded::<ConnEvent>();

    // Writer task: wait until the socket is OPEN, then drain outgoing.
    // Sending while CONNECTING throws on some browsers; doing it here
    // used to kill the writer before it ever opened.
    let ws_writer = ws.clone();
    let conn_tx_writer = conn_tx.clone();
    spawn_local(async move {
        loop {
            match ws_writer.ready_state() {
                WebSocket::OPEN => break,
                WebSocket::CLOSING | WebSocket::CLOSED => {
                    let _ = conn_tx_writer
                        .unbounded_send(ConnEvent::Errored("socket closed before open".into()));
                    return;
                }
                _ => {
                    gloo_timers::future::TimeoutFuture::new(10).await;
                }
            }
        }
        let mut outgoing = outgoing;
        while let Some(msg) = outgoing.next().await {
            let json = match message_to_json_str(&msg) {
                Ok(j) => j,
                Err(e) => {
                    web_sys::console::error_1(&format!("serialize Message: {e:?}").into());
                    continue;
                }
            };
            if ws_writer.send_with_str(&json).is_err() {
                web_sys::console::warn_1(&"ws send failed, closing writer".into());
                let _ = conn_tx_writer
                    .unbounded_send(ConnEvent::Errored("send failed".into()));
                return;
            }
        }
    });

    let onmessage = {
        let tx = incoming_responses_tx.clone();
        Closure::<dyn FnMut(MessageEvent)>::new(move |e: MessageEvent| {
            let Some(text) = e.data().as_string() else {
                web_sys::console::warn_1(&"ws message wasn't a string".into());
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

    let onopen = {
        let tx = conn_tx.clone();
        Closure::<dyn FnMut(JsValue)>::new(move |_| {
            let _ = tx.unbounded_send(ConnEvent::Opened);
        })
    };
    ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));

    let onclose = {
        let tx = conn_tx.clone();
        Closure::<dyn FnMut(JsValue)>::new(move |_| {
            let _ = tx.unbounded_send(ConnEvent::Closed("socket closed".into()));
        })
    };
    ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));

    let onerror = {
        let tx = conn_tx.clone();
        Closure::<dyn FnMut(JsValue)>::new(move |_| {
            let _ = tx.unbounded_send(ConnEvent::Errored("socket error".into()));
        })
    };
    ws.set_onerror(Some(onerror.as_ref().unchecked_ref()));

    Ok(ClientHandle {
        router,
        conn_events: conn_rx,
        ws,
        _onmessage: Some(onmessage),
        _onopen: Some(onopen),
        _onclose: Some(onclose),
        _onerror: Some(onerror),
    })
}
