use crate::logging::{log, log_error, request_name};
use crate::reply::{craft_response, send_message};
use crate::telephone::{Telephone, TelephoneCommand};
use axum::extract::ws::{Message, WebSocket};
use server_errors::{ErrorDetails, LogCategory, ServErr};

use protocol::serialization::Convert;
use queue::{Priority, QueueManager, WhereTheRequestCameFrom};
use server_protocol::{
    GetMyUnsentLettersPayload, LoginReply, Request, Response, json_str_to_message,
    response_to_json_str,
};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::mpsc::UnboundedReceiver;

static NEXT_SOCKET_ID: AtomicU64 = AtomicU64::new(1);

enum NextRequest {
    Got(Request, usize, Vec<u8>, Option<String>),
    BadProtocol(ServErr),
    Closed,
}

/// Everything one websocket connection carries between loop turns.
struct SocketState {
    socket: WebSocket,
    socket_id: u64,
    /// Who this socket is, for log lines. The address until login renames it.
    label: String,
    /// Created at birth, handed to the telephone at birth. Never moves.
    /// The telephone writes a `TelephoneCommand` here; on receipt, the
    /// socket acts on it (pull, or send a drift-changed message and
    /// close).
    notify: UnboundedReceiver<TelephoneCommand>,
}

impl SocketState {
    fn new(
        socket: WebSocket,
        socket_id: u64,
        addr: SocketAddr,
        notify: UnboundedReceiver<TelephoneCommand>,
    ) -> Self {
        SocketState {
            socket,
            socket_id,
            label: addr.to_string(),
            notify,
        }
    }

    async fn run(&mut self, telephone: &Telephone, queue: &QueueManager) {
        log(&self.label, "connected");

        loop {
            tokio::select! {
                next = Self::next_request(&mut self.socket) => {
                    let (request, msg_id, payload, drift_version) = match next {
                        NextRequest::Got(r, i, p, d) => (r, i, p, d),
                        NextRequest::BadProtocol(e) => {
                            self.reject_bad_protocol(e).await;
                            continue;
                        }
                        NextRequest::Closed => break,
                    };

                    log(&self.label, request_name(&request));

                    let is_login = matches!(request, Request::Login);

                    let Some(bytes) = self
                        .run_request(request.clone(), msg_id, payload, drift_version, queue)
                        .await
                    else {
                        continue;
                    };

                    if is_login {
                        if let Some(name) = telephone.user_of(self.socket_id).await {
                            self.label = name;
                        }
                    }

                    if !self.send_reply(bytes, msg_id, request).await {
                        break;
                    }
                }

                notified = self.notify.recv() => {
                    let Some(command) = notified else {
                        panic!("the telephone's task ended while sockets are still connected");
                    };
                    match command {
                        TelephoneCommand::GoPullYourUnsentLetters => {
                            if !self.pull_and_push(queue).await {
                                break;
                            }
                        }
                        TelephoneCommand::TellTheClientTheDriftVersionChangedThenClose => {
                            self.tell_the_client_the_drift_version_changed_then_close(queue)
                                .await;
                            break;
                        }
                    }
                }
            }
        }

        log(&self.label, "disconnected");
    }

    /// Submit a request to the queue on the socket's own behalf (not for
    /// a client), await the reply, and hand back the raw bytes. Logs on
    /// either failure. `log_prefix` shows up in the log lines so callers
    /// are tellable apart.
    async fn make_request_from_the_socket_directly_and_await_reply(
        &mut self,
        request: Request,
        queue: &QueueManager,
        log_prefix: &str,
    ) -> Option<Vec<u8>> {
        // message_id 0 for internal requests: nothing on the client side
        // matches it, and it never reaches the client.
        let reply_rx = match queue.submit(
            0,
            request,
            Vec::new(),
            self.socket_id,
            WhereTheRequestCameFrom::FromTheServer,
            Priority::Normal,
        ) {
            Ok(rx) => rx,
            Err(e) => {
                log_error(
                    &self.label,
                    LogCategory::Queue,
                    &ErrorDetails {
                        err_msg: format!("{log_prefix}: couldn't queue: {e}"),
                        file: file!().to_string(),
                        method: "SocketState::make_request_from_the_socket_directly_and_await_reply"
                            .to_string(),
                        probable_cause: "the queue executor isn't running".to_string(),
                    },
                );
                return None;
            }
        };

        match reply_rx.await {
            Ok(bytes) => Some(bytes),
            Err(_) => {
                log_error(
                    &self.label,
                    LogCategory::Queue,
                    &ErrorDetails {
                        err_msg: format!("{log_prefix}: queue dropped the reply"),
                        file: file!().to_string(),
                        method: "SocketState::make_request_from_the_socket_directly_and_await_reply"
                            .to_string(),
                        probable_cause: "the executor task ended".to_string(),
                    },
                );
                None
            }
        }
    }

    /// The socket was told "go pull". Submit GetMyUnsentLetters through
    /// the queue for this socket's user, then send each letter back to
    /// the client as its own `LoveLetterFromServer` message, so each one
    /// can be applied and acked on its own.
    ///
    /// Returns true if the socket is still usable. A failed pull (queue
    /// closed, decode error) is logged and treated as "try again next
    /// time"; the letters are still rowed server-side.
    async fn pull_and_push(&mut self, queue: &QueueManager) -> bool {
        let Some(bytes) = self
            .make_request_from_the_socket_directly_and_await_reply(
                Request::GetMyUnsentLetters,
                queue,
                "pull",
            )
            .await
        else {
            return true;
        };

        let outer: Result<GetMyUnsentLettersPayload, ServErr> =
            match Convert::un_payloadify(&bytes) {
                Ok(v) => v,
                Err(e) => {
                    log_error(
                        &self.label,
                        LogCategory::LetterDelivery,
                        &ErrorDetails {
                            err_msg: format!("pull: couldn't decode payload: {e:?}"),
                            file: file!().to_string(),
                            method: "SocketState::pull_and_push".to_string(),
                            probable_cause: String::new(),
                        },
                    );
                    return true;
                }
            };

        let payload = match outer {
            Ok(p) => p,
            Err(e) => {
                log_error(
                    &self.label,
                    LogCategory::LetterDelivery,
                    &ErrorDetails {
                        err_msg: format!("pull: server returned error: {e:?}"),
                        file: file!().to_string(),
                        method: "SocketState::pull_and_push".to_string(),
                        probable_cause: String::new(),
                    },
                );
                return true;
            }
        };

        for letter in payload.letters {
            let response = Response::LoveLetterFromServer {
                letter,
                drift_version: payload.drift_version.clone(),
            };
            if !self.send_response_to_client(response, "push").await {
                return false;
            }
        }
        true
    }

    /// The telephone told this socket its drift version is stale. Ask
    /// the executor what the current one is, send the client a message
    /// saying so, and return. The caller breaks the loop, which closes
    /// the websocket.
    async fn tell_the_client_the_drift_version_changed_then_close(
        &mut self,
        queue: &QueueManager,
    ) {
        let Some(bytes) = self
            .make_request_from_the_socket_directly_and_await_reply(
                Request::GetCurrentDrift,
                queue,
                "drift-changed",
            )
            .await
        else {
            return;
        };

        let outer: Result<LoginReply, ServErr> = match Convert::un_payloadify(&bytes) {
            Ok(v) => v,
            Err(e) => {
                log_error(
                    &self.label,
                    LogCategory::LetterDelivery,
                    &ErrorDetails {
                        err_msg: format!("drift-changed: couldn't decode payload: {e:?}"),
                        file: file!().to_string(),
                        method:
                            "SocketState::tell_the_client_the_drift_version_changed_then_close"
                                .to_string(),
                        probable_cause: String::new(),
                    },
                );
                return;
            }
        };

        let reply = match outer {
            Ok(r) => r,
            Err(e) => {
                log_error(
                    &self.label,
                    LogCategory::LetterDelivery,
                    &ErrorDetails {
                        err_msg: format!("drift-changed: server returned error: {e:?}"),
                        file: file!().to_string(),
                        method:
                            "SocketState::tell_the_client_the_drift_version_changed_then_close"
                                .to_string(),
                        probable_cause: String::new(),
                    },
                );
                return;
            }
        };

        let response = Response::TheDriftVersionChanged {
            drift_version: reply.drift_version,
            drift_reason: reply.drift_reason,
        };
        let _ = self.send_response_to_client(response, "drift-changed").await;
    }

    async fn next_request(socket: &mut WebSocket) -> NextRequest {
        loop {
            match socket.recv().await {
                Some(Ok(Message::Text(text))) => {
                    return match json_str_to_message(&text) {
                        Ok(m) => NextRequest::Got(m.request, m.message_id, m.payload.0, m.drift_version),
                        Err(e) => NextRequest::BadProtocol(e),
                    };
                }
                Some(Ok(_)) => continue,
                _ => return NextRequest::Closed,
            }
        }
    }

    async fn reject_bad_protocol(&mut self, e: ServErr) {
        log(&self.label, format!("bad message: {e}"));
        if let Err(e2) = self
            .socket
            .send(Message::Text("Message doesn't use correct protocol".into()))
            .await
        {
            log(&self.label, format!("send protocol error failed: {e2}"));
        }
    }

    async fn run_request(
        &mut self,
        request: Request,
        msg_id: usize,
        payload: Vec<u8>,
        drift_version: Option<String>,
        queue: &QueueManager,
    ) -> Option<Vec<u8>> {
        let reply_rx = match queue.submit(
            msg_id,
            request.clone(),
            payload,
            self.socket_id,
            WhereTheRequestCameFrom::FromAClient(drift_version),
            Priority::Normal,
        ) {
            Ok(rx) => rx,
            Err(e) => {
                log_error(
                    &self.label,
                    LogCategory::Queue,
                    &ErrorDetails {
                        err_msg: format!("couldn't queue the request: {e}"),
                        file: file!().to_string(),
                        method: "SocketState::run_request".to_string(),
                        probable_cause: "the queue executor isn't running".to_string(),
                    },
                );
                let err: Result<(), ServErr> = Err(ServErr::ServerError(ErrorDetails {
                    err_msg: format!("queue closed: {e}"),
                    file: file!().to_string(),
                    method: "SocketState::run_request".to_string(),
                    probable_cause: "the queue executor isn't running".to_string(),
                }));
                self.send_reply(err.to_payload(), msg_id, request).await;
                return None;
            }
        };

        match reply_rx.await {
            Ok(bytes) => Some(bytes),
            Err(_) => {
                log_error(
                    &self.label,
                    LogCategory::Queue,
                    &ErrorDetails {
                        err_msg: "the queue dropped the reply".to_string(),
                        file: file!().to_string(),
                        method: "SocketState::run_request".to_string(),
                        probable_cause: "the executor task ended".to_string(),
                    },
                );
                let err: Result<(), ServErr> = Err(ServErr::ServerError(ErrorDetails {
                    err_msg: "queue dropped the reply".to_string(),
                    file: file!().to_string(),
                    method: "SocketState::run_request".to_string(),
                    probable_cause: "the executor task ended".to_string(),
                }));
                self.send_reply(err.to_payload(), msg_id, request).await;
                None
            }
        }
    }

    async fn send_reply(&mut self, bytes: Vec<u8>, msg_id: usize, request: Request) -> bool {
        let response = craft_response(bytes, msg_id, request);
        match send_message(&mut self.socket, response).await {
            Ok(()) => true,
            Err(e) => {
                log(&self.label, format!("send response failed: {e}"));
                false
            }
        }
    }

    /// Serialize one response and send it to the client. Logs on either
    /// failure. `log_prefix` shows up in every log line so "push" and
    /// "drift-changed" are tellable apart. Returns false only if the
    /// socket send itself failed; a serialize failure isn't a reason to
    /// drop the connection.
    async fn send_response_to_client(&mut self, response: Response, log_prefix: &str) -> bool {
        let json = match response_to_json_str(&response) {
            Ok(j) => j,
            Err(e) => {
                log_error(
                    &self.label,
                    LogCategory::LetterDelivery,
                    &ErrorDetails {
                        err_msg: format!("{log_prefix}: serialize failed: {e}"),
                        file: file!().to_string(),
                        method: "SocketState::send_response_to_client".to_string(),
                        probable_cause: String::new(),
                    },
                );
                return true;
            }
        };

        match self.socket.send(Message::Text(json.into())).await {
            Ok(()) => true,
            Err(e) => {
                log_error(
                    &self.label,
                    LogCategory::LetterDelivery,
                    &ErrorDetails {
                        err_msg: format!("{log_prefix}: send failed: {e}"),
                        file: file!().to_string(),
                        method: "SocketState::send_response_to_client".to_string(),
                        probable_cause: "the client's socket closed".to_string(),
                    },
                );
                false
            }
        }
    }
}

pub async fn socket_main(
    socket: WebSocket,
    telephone: Telephone,
    queue: QueueManager,
    addr: SocketAddr,
) {
    let socket_id = NEXT_SOCKET_ID.fetch_add(1, Ordering::Relaxed);
    let (notify_tx, notify_rx) = tokio::sync::mpsc::unbounded_channel();
    telephone.register_socket(socket_id, notify_tx);

    let mut state = SocketState::new(socket, socket_id, addr, notify_rx);
    state.run(&telephone, &queue).await;

    telephone.remove_socket(socket_id);
}