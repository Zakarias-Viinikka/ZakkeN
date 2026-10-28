//! Android socket half. Not implemented yet; the module exists so
//! `Router::new` compiles for the android target. `spawn` will follow
//! the same signature as `socket_web::spawn` once a websocket crate is
//! picked and a tokio runtime is wired up.

use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};
use server_protocol::{Message, Response};

use crate::conn_status::ConnStatus;

pub(crate) fn spawn<S>(
    _url: String,
    _outgoing_rx: UnboundedReceiver<Message>,
    _incoming_tx: UnboundedSender<Response>,
    _on_conn_status: S,
) where
    S: FnMut(ConnStatus) + 'static,
{
    unimplemented!("android socket not wired yet");
}

pub(crate) fn spawn_dispatch(_router: crate::router::Router) {
    unimplemented!("android socket not wired yet");
}
