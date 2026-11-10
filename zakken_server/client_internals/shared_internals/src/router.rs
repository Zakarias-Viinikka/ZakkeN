//! The router owns the socket connection to `zakken_server`.
//!
//! `Router::new(url, tx_forward_spontaneous_server_message, on_conn_status)`
//! creates the shared state, spawns the dispatch loop and the
//! socket-retry loop, and returns a `RouterHandle`. Callers only ever
//! see `RouterHandle`.
//!
//! The `Router` struct itself is the dispatch loop: it reads decoded
//! responses from the socket half, matches `Reply { message_id }` to
//! the waiter that asked, and forwards every spontaneous response
//! (love letters, the drift-changed push) into the caller-supplied
//! channel. The owner of spontaneous messages lives on the other end
//! of that channel and decides what each one means.

use server_errors::{ErrorDetails, ServErr};
use futures::StreamExt;
#[allow(unused_imports)]
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};
use futures::channel::oneshot;
use protocol::serialization::Base64Bytes;
use server_protocol::{Message, Request, Response, UnsentLetter};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

#[cfg(any(target_arch = "wasm32", target_os = "android"))]
use crate::conn_status::ConnStatus;

#[cfg(target_arch = "wasm32")]
use crate::socket_web as socket;

#[cfg(target_os = "android")]
use crate::socket_android as socket;

struct Shared {
    pending: Mutex<HashMap<usize, oneshot::Sender<Vec<u8>>>>,
    next_request_id: AtomicUsize,
    outgoing_tx: UnboundedSender<Message>,
    connected: AtomicBool,
}

#[cfg_attr(
    not(any(target_arch = "wasm32", target_os = "android")),
    allow(dead_code)
)]
pub struct Router {
    shared: Arc<Shared>,
    incoming_responses_rx: UnboundedReceiver<Response>,
    tx_forward_spontaneous_server_message: UnboundedSender<Response>,
}

impl Router {
    /// Create the shared state, spawn the dispatch loop and the socket
    /// retry loop, return a handle.
    ///
    /// - `tx_forward_spontaneous_server_message` carries every
    ///   spontaneous response the server pushes.
    /// - `on_conn_status` fires on every connection state change.

    #[cfg(any(target_arch = "wasm32", target_os = "android"))]
    pub fn new<S>(
        url: String,
        tx_forward_spontaneous_server_message: UnboundedSender<Response>,
        on_conn_status: S,
    ) -> RouterHandle
    where
        S: FnMut(ConnStatus) + 'static,
    {
        let (outgoing_tx, outgoing_rx) = unbounded::<Message>();
        let (incoming_tx, incoming_rx) = unbounded::<Response>();

        let shared = Arc::new(Shared {
            pending: Mutex::new(HashMap::new()),
            next_request_id: AtomicUsize::new(1),
            outgoing_tx,
            connected: AtomicBool::new(false),
        });

        let dispatch = Router {
            shared: Arc::clone(&shared),
            incoming_responses_rx: incoming_rx,
            tx_forward_spontaneous_server_message,
        };
        socket::spawn_dispatch(dispatch);

        // Whenever the socket dies, drop every waiter still hoping for
        // a reply over the old connection. The server does not replay
        // in-flight replies on reconnect, so without this the waiting
        // callers would hang forever.
        let shared_for_disconnect = Arc::clone(&shared);
        let on_disconnect = move || {
            shared_for_disconnect.pending.lock().unwrap().clear();
        };
        let shared_for_conn = Arc::clone(&shared);
        let mut on_conn_status_from_caller = on_conn_status;
        let on_conn_status = move |status: ConnStatus| {
            shared_for_conn
                .connected
                .store(matches!(status, ConnStatus::Connected), Ordering::SeqCst);
            on_conn_status_from_caller(status);
        };
        socket::spawn(
            url,
            outgoing_rx,
            incoming_tx,
            on_conn_status,
            on_disconnect,
        );

        RouterHandle { shared }
    }

    /// The dispatch loop. Ends when the socket half drops
    /// `incoming_tx`, i.e. when the socket task fully exits.
    #[cfg_attr(
        not(any(target_arch = "wasm32", target_os = "android")),
        allow(dead_code)
    )]
    pub(crate) async fn run(mut self) {
        while let Some(response) = self.incoming_responses_rx.next().await {
            match response {
                Response::Reply {
                    message_id,
                    payload,
                    ..
                } => {
                    let waiter = self.shared.pending.lock().unwrap().remove(&message_id);
                    if let Some(tx) = waiter {
                        let _ = tx.send(payload.0);
                    }
                }
                Response::LoveLetterFromServer { .. } | Response::TheDriftVersionChanged { .. } => {
                    let _ = self
                        .tx_forward_spontaneous_server_message
                        .unbounded_send(response);
                }
            }
        }
    }
}

/// The client-facing side. Cheap to clone. Both this and the running
/// dispatch task share the same state, so a clone keeps the connection
/// alive.
#[derive(Clone)]
pub struct RouterHandle {
    shared: Arc<Shared>,
}

impl RouterHandle {
    /// Send a request and await the reply's payload bytes.
    pub async fn send_request(
        &self,
        request: Request,
        payload: Vec<u8>,
    ) -> Result<Vec<u8>, ServErr> {
        if !self.shared.connected.load(Ordering::SeqCst) {
            return Err(ServErr::ServerError(ErrorDetails {
                err_msg: "not connected to the server".to_string(),
                file: file!().to_string(),
                method: "send_request".to_string(),
                probable_cause: "the socket isn't open".to_string(),
            }));
        }
        let id = self.shared.next_request_id.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = oneshot::channel();

        self.shared.pending.lock().unwrap().insert(id, tx);

        let message = Message {
            message_id: id,
            request,
            payload: Base64Bytes(payload),
            drift_version: crate::drift_version::get_drift_version(),
        };
        if self.shared.outgoing_tx.unbounded_send(message).is_err() {
            self.shared.pending.lock().unwrap().remove(&id);
            return Err(ServErr::ServerError(ErrorDetails {
                err_msg: "outgoing channel is closed".to_string(),
                file: file!().to_string(),
                method: "send_request".to_string(),
                probable_cause: "the router is shutting down".to_string(),
            }));
        }

        rx.await.map_err(|_| {
            ServErr::ServerError(ErrorDetails {
                err_msg: "reply was dropped before it arrived".to_string(),
                file: file!().to_string(),
                method: "send_request".to_string(),
                probable_cause: "the connection closed".to_string(),
            })
        })
    }

    /// Drop every pending waiter. Their `await` resolves with an error.
    pub fn fail_all_pending(&self) {
        self.shared.pending.lock().unwrap().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use server_protocol::Request;

    fn reply(id: usize, bytes: Vec<u8>) -> Response {
        Response::Reply {
            message_id: id,
            request: Request::Ack,
            payload: Base64Bytes(bytes),
        }
    }

    // Builds the shared state + dispatch loop + handle without touching
    // the socket. Tests drive the channel ends directly.
    fn make() -> (
        RouterHandle,
        Router,
        UnboundedSender<Response>,
        UnboundedReceiver<Message>,
        UnboundedReceiver<Response>,
    ) {
        let (outgoing_tx, outgoing_rx) = unbounded::<Message>();
        let (incoming_tx, incoming_rx) = unbounded::<Response>();
        let (spontaneous_tx, spontaneous_rx) = unbounded::<Response>();
        let shared = Arc::new(Shared {
            pending: Mutex::new(HashMap::new()),
            next_request_id: AtomicUsize::new(1),
            outgoing_tx,
            connected: AtomicBool::new(true),
        });
        let dispatch = Router {
            shared: Arc::clone(&shared),
            incoming_responses_rx: incoming_rx,
            tx_forward_spontaneous_server_message: spontaneous_tx,
        };
        (
            RouterHandle { shared },
            dispatch,
            incoming_tx,
            outgoing_rx,
            spontaneous_rx,
        )
    }

    #[test]
    fn a_reply_goes_to_the_caller_that_asked() {
        let (handle, dispatch, incoming_tx, mut outgoing_rx, _spontaneous_rx) = make();

        block_on(async move {
            let call = handle.send_request(Request::Ack, Vec::new());
            let run = dispatch.run();

            let drive = async {
                let sent = outgoing_rx.next().await.expect("router sent one");
                assert_eq!(sent.message_id, 1);
                incoming_tx
                    .unbounded_send(reply(sent.message_id, vec![9, 8, 7]))
                    .expect("feed a response");
                drop(incoming_tx);
            };

            let (_, _, got) = futures::join!(run, drive, call);
            assert_eq!(got.expect("payload"), vec![9, 8, 7]);
        });
    }

    #[test]
    fn a_love_letter_from_server_is_forwarded_unchanged() {
        let (_handle, dispatch, incoming_tx, _outgoing_rx, mut spontaneous_rx) = make();
        let letter = UnsentLetter {
            letter_id: 5,
            love_letter: vec![1, 2, 3],
        };

        block_on(async move {
            let run = dispatch.run();
            incoming_tx
                .unbounded_send(Response::LoveLetterFromServer {
                    letter: letter.clone(),
                    drift_version: "test".to_string(),
                })
                .expect("feed a push");
            drop(incoming_tx);

            let drive = async {
                let got = spontaneous_rx.next().await.expect("one forwarded");
                match got {
                    Response::LoveLetterFromServer { letter: got, drift_version } => {
                        assert_eq!(got.letter_id, 5);
                        assert_eq!(drift_version, "test");
                    }
                    _ => panic!("wrong variant"),
                }
            };

            let _ = futures::join!(run, drive);
        });
    }

    #[test]
    fn the_drift_changed_push_is_forwarded_unchanged() {
        let (_handle, dispatch, incoming_tx, _outgoing_rx, mut spontaneous_rx) = make();

        block_on(async move {
            let run = dispatch.run();
            incoming_tx
                .unbounded_send(Response::TheDriftVersionChanged {
                    drift_version: "v2".to_string(),
                    drift_reason: "another client wiped".to_string(),
                })
                .expect("feed a push");
            drop(incoming_tx);

            let drive = async {
                let got = spontaneous_rx.next().await.expect("one forwarded");
                match got {
                    Response::TheDriftVersionChanged { drift_version, drift_reason } => {
                        assert_eq!(drift_version, "v2");
                        assert_eq!(drift_reason, "another client wiped");
                    }
                    _ => panic!("wrong variant"),
                }
            };

            let _ = futures::join!(run, drive);
        });
    }
}
