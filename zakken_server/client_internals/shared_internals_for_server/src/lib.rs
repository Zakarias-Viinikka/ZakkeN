//! Shared client internals for talking to a zakken_server.
//!
//! The router is target-agnostic. It owns no socket. It gives each
//! request a fresh id and hands the payload of the matching reply back
//! to whoever asked.
//!
//! The server also pushes love letters with no id (the
//! `Response::LoveLetterFromServer` variant). Those don't belong to any
//! request, so the router forwards them to a callback the app supplies
//! at construction. What the app does with them (apply and ack, in the
//! real client) is the app's business.

use error_stuff::{ErrorDetails, ServErr};
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::channel::oneshot;
use futures::StreamExt;
use server_protocol::{Message, Response, UnsentLetter};
use std::collections::HashMap;
use std::future::Future;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct ResponseRouter {
    outgoing_tx: UnboundedSender<Message>,
    next_request_id: Arc<AtomicUsize>,
    pending: Arc<Mutex<HashMap<usize, oneshot::Sender<Vec<u8>>>>>,
}

/// The channels the caller needs to wire up the socket side.
pub struct RouterHandles {
    /// Messages the router wants sent. The socket task reads this and
    /// writes each one to the wire.
    pub outgoing: UnboundedReceiver<Message>,
    /// The socket task puts every decoded response here. The router
    /// reads this.
    pub incoming_responses_tx: UnboundedSender<Response>,
}

impl ResponseRouter {
    /// Make a router and the channels its socket half needs. Also
    /// returns a future that must be spawned: the loop that reads
    /// incoming responses, resolves waiters, and invokes
    /// `on_love_letters` for server pushes.
    pub fn new<F>(
        on_love_letters: F,
    ) -> (Self, RouterHandles, impl Future<Output = ()>)
    where
        F: FnMut(Vec<UnsentLetter>) + 'static,
    {
        let (outgoing_tx, outgoing_rx) = unbounded::<Message>();
        let (incoming_tx, incoming_rx) = unbounded::<Response>();

        let pending = Arc::new(Mutex::new(
            HashMap::<usize, oneshot::Sender<Vec<u8>>>::new(),
        ));
        let pending_clone = Arc::clone(&pending);

        let router = Self {
            outgoing_tx,
            next_request_id: Arc::new(AtomicUsize::new(1)),
            pending,
        };

        let routing = async move {
            let mut on_love_letters = on_love_letters;
            let mut incoming = incoming_rx;
            while let Some(response) = incoming.next().await {
                match response {
                    Response::Reply { message_id, payload, .. } => {
                        let waiter = pending_clone
                            .lock()
                            .unwrap()
                            .remove(&message_id);
                        if let Some(tx) = waiter {
                            let _ = tx.send(payload.0);
                        }
                        // No waiter: a reply the caller gave up on.
                        // Drop it.
                    }
                    Response::LoveLetterFromServer(letters) => {
                        on_love_letters(letters);
                    }
                }
            }
        };

        (
            router,
            RouterHandles {
                outgoing: outgoing_rx,
                incoming_responses_tx: incoming_tx,
            },
            routing,
        )
    }

    /// Build a Message with a fresh id and send it. The returned
    /// receiver resolves with the reply's payload bytes when they
    /// arrive.
    pub fn send_request<F>(
        &self,
        build_message: F,
    ) -> Result<oneshot::Receiver<Vec<u8>>, ServErr>
    where
        F: FnOnce(usize) -> Message,
    {
        let id = self.next_request_id.fetch_add(1, Ordering::Relaxed);
        let message = build_message(id);

        let (reply_tx, reply_rx) = oneshot::channel();
        self.pending.lock().unwrap().insert(id, reply_tx);

        if self.outgoing_tx.unbounded_send(message).is_err() {
            self.pending.lock().unwrap().remove(&id);
            return Err(ServErr::ServerError(ErrorDetails {
                err_msg: "outgoing channel to the socket half is closed".to_string(),
                file: file!().to_string(),
                method: "send_request".to_string(),
                probable_cause: "the socket task ended".to_string(),
            }));
        }

        Ok(reply_rx)
    }

    /// Drop every pending waiter. Their `await` returns `Err(RecvError)`.
    /// Called when the socket closes, so callers don't hang forever.
    pub fn fail_all_pending(&self) {
        self.pending.lock().unwrap().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use protocol::serialization::Base64Bytes;
    use server_protocol::Request;

    fn msg(id: usize) -> Message {
        Message {
            message_id: id,
            request: Request::Ack,
            payload: Base64Bytes(Vec::new()),
        }
    }

    fn reply(id: usize, bytes: Vec<u8>) -> Response {
        Response::Reply {
            message_id: id,
            request: Request::Ack,
            payload: Base64Bytes(bytes),
        }
    }

    #[test]
    fn a_reply_goes_to_the_caller_that_asked() {
        let (router, mut handles, routing) = ResponseRouter::new(|_| {});

        block_on(async move {
            let reply_rx = router.send_request(msg).expect("send_request ok");
            let sent = handles.outgoing.next().await.expect("router sent one");
            assert_eq!(sent.message_id, 1);

            handles
                .incoming_responses_tx
                .unbounded_send(reply(sent.message_id, vec![9, 8, 7]))
                .expect("feed a response");
            drop(handles.incoming_responses_tx);

            let (_, got) = futures::join!(routing, reply_rx);
            assert_eq!(got.expect("payload"), vec![9, 8, 7]);
        });
    }

    #[test]
    fn a_love_letter_from_server_goes_to_the_callback() {
        let received: Arc<Mutex<Vec<Vec<UnsentLetter>>>> = Arc::new(Mutex::new(Vec::new()));
        let received_clone = Arc::clone(&received);
        let on_letters = move |letters: Vec<UnsentLetter>| {
            received_clone.lock().unwrap().push(letters);
        };

        let (_router, handles, routing) = ResponseRouter::new(on_letters);

        let letter = UnsentLetter {
            letter_id: 5,
            love_letter: vec![1, 2, 3],
        };

        block_on(async move {
            handles
                .incoming_responses_tx
                .unbounded_send(Response::LoveLetterFromServer(vec![letter]))
                .expect("feed a push");
            drop(handles.incoming_responses_tx);
            routing.await;
        });

        let got = received.lock().unwrap();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].len(), 1);
        assert_eq!(got[0][0].letter_id, 5);
    }
}