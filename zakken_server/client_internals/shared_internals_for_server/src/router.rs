//! The router owns the socket connection to `zakken_server`.
//!
//! `Router::new(url, on_love_letters, on_conn_status)` creates the
//! shared state, spawns the dispatch loop and the socket-retry loop,
//! and returns a `RouterHandle`. Callers only ever see `RouterHandle`.
//!
//! The `Router` struct itself is the dispatch loop: it reads decoded
//! responses from the socket half, matches `Reply { message_id }` to
//! the waiter that asked, and forwards `LoveLetterFromServer` to the
//! caller-supplied callback. It's spawned inside `new`, not returned.

use error_stuff::{ErrorDetails, ServErr};
#[allow(unused_imports)]
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::channel::oneshot;
use futures::StreamExt;
use protocol::serialization::Base64Bytes;
use server_protocol::{Message, Request, Response, UnsentLetter};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
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
}

pub struct Router {
    shared: Arc<Shared>,
    incoming_responses_rx: UnboundedReceiver<Response>,
    on_love_letters: Box<dyn FnMut(UnsentLetter)>,
}

impl Router {
    /// Create the shared state, spawn the dispatch loop and the socket
    /// retry loop, return a handle.
    ///
    /// - `on_love_letters` fires for every server-pushed letter.
    /// - `on_conn_status` fires on every connection state change.

    #[cfg(any(target_arch = "wasm32", target_os = "android"))]
    pub fn new<L, S>(url: String, on_love_letters: L, on_conn_status: S) -> RouterHandle
    where
        L: FnMut(UnsentLetter) + 'static,
        S: FnMut(ConnStatus) + 'static,
    {
        let (outgoing_tx, outgoing_rx) = unbounded::<Message>();
        let (incoming_tx, incoming_rx) = unbounded::<Response>();

        let shared = Arc::new(Shared {
            pending: Mutex::new(HashMap::new()),
            next_request_id: AtomicUsize::new(1),
            outgoing_tx,
        });

        let dispatch = Router {
            shared: Arc::clone(&shared),
            incoming_responses_rx: incoming_rx,
            on_love_letters: Box::new(on_love_letters),
        };
        socket::spawn_dispatch(dispatch);
        socket::spawn(url, outgoing_rx, incoming_tx, on_conn_status);

        RouterHandle { shared }
    }

    /// The dispatch loop. Ends when the socket half drops
    /// `incoming_tx`, i.e. when the socket task fully exits.
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
                Response::LoveLetterFromServer(letter) => {
                    (self.on_love_letters)(letter);
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
        let id = self.shared.next_request_id.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = oneshot::channel();

        self.shared.pending.lock().unwrap().insert(id, tx);

        let message = Message {
            message_id: id,
            request,
            payload: Base64Bytes(payload),
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
    // the socket. Tests drive the two channel ends directly.
    fn make<L>(
        on_letters: L,
    ) -> (
        RouterHandle,
        Router,
        UnboundedSender<Response>,
        UnboundedReceiver<Message>,
    )
    where
        L: FnMut(UnsentLetter) + 'static,
    {
        let (outgoing_tx, outgoing_rx) = unbounded::<Message>();
        let (incoming_tx, incoming_rx) = unbounded::<Response>();
        let shared = Arc::new(Shared {
            pending: Mutex::new(HashMap::new()),
            next_request_id: AtomicUsize::new(1),
            outgoing_tx,
        });
        let dispatch = Router {
            shared: Arc::clone(&shared),
            incoming_responses_rx: incoming_rx,
            on_love_letters: Box::new(on_letters),
        };
        (RouterHandle { shared }, dispatch, incoming_tx, outgoing_rx)
    }

    #[test]
    fn a_reply_goes_to_the_caller_that_asked() {
        let (handle, dispatch, incoming_tx, mut outgoing_rx) = make(|_| {});

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
    fn a_love_letter_from_server_goes_to_the_callback() {
        let received: Arc<Mutex<Vec<UnsentLetter>>> = Arc::new(Mutex::new(Vec::new()));
        let received_clone = Arc::clone(&received);
        let on_letters = move |letter: UnsentLetter| {
            received_clone.lock().unwrap().push(letter);
        };

        let (_handle, dispatch, incoming_tx, _outgoing_rx) = make(on_letters);
        let letter = UnsentLetter {
            letter_id: 5,
            love_letter: vec![1, 2, 3],
        };

        block_on(async move {
            let run = dispatch.run();
            incoming_tx
                .unbounded_send(Response::LoveLetterFromServer(letter))
                .expect("feed a push");
            drop(incoming_tx);
            run.await;
        });

        let got = received.lock().unwrap();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].letter_id, 5);
    }
}