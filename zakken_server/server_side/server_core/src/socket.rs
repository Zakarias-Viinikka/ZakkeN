use crate::logging::{log, log_error, request_name};
use crate::reply::{craft_response, send_message};
use crate::telephone::Telephone;
use axum::extract::ws::{Message, WebSocket};
use error_stuff::{ErrorDetails, LogCategory, ServErr};
use protocol::error::DbError;
use protocol::serialization::{Base64Bytes, Convert};
use queue::{Priority, QueueManager};
use server_protocol::{Request, Response, json_str_to_message, response_to_json_str};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::mpsc::UnboundedReceiver;

static NEXT_SOCKET_ID: AtomicU64 = AtomicU64::new(1);

enum NextRequest {
    Got(Request, usize, Vec<u8>),
    BadProtocol(ServErr),
    Closed,
}

struct SocketSession {
    socket: WebSocket,
    socket_id: u64,
    /// Who this socket is, for log lines. The address until login renames it.
    label: String,
    /// Created at birth, handed to the telephone at birth. Never moves.
    incoming_nudges: UnboundedReceiver<()>,
}

impl SocketSession {
    fn new(
        socket: WebSocket,
        socket_id: u64,
        addr: SocketAddr,
        incoming_nudges: UnboundedReceiver<()>,
    ) -> Self {
        SocketSession {
            socket,
            socket_id,
            label: addr.to_string(),
            incoming_nudges,
        }
    }

    async fn run(&mut self, telephone: &Telephone, queue: &QueueManager) {
        log(&self.label, "connected");

        loop {
            tokio::select! {
                next = Self::next_request(&mut self.socket) => {
                    let (request, msg_id, payload) = match next {
                        NextRequest::Got(r, i, p) => (r, i, p),
                        NextRequest::BadProtocol(e) => {
                            self.reject_bad_protocol(e).await;
                            continue;
                        }
                        NextRequest::Closed => break,
                    };

                    log(&self.label, request_name(&request));

                    let is_login = matches!(request, Request::Login);

                    let Some(bytes) = self
                        .run_request(request.clone(), msg_id, payload, queue)
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

                nudge = self.incoming_nudges.recv() => {
                    if nudge.is_none() {
                        // The telephone's task ended; no more nudges will
                        // come. Stop this socket too.
                        break;
                    }
                    if !self.send_nudge().await {
                        break;
                    }
                }
            }
        }

        log(&self.label, "disconnected");
    }

    async fn next_request(socket: &mut WebSocket) -> NextRequest {
        loop {
            match socket.recv().await {
                Some(Ok(Message::Text(text))) => {
                    return match json_str_to_message(&text) {
                        Ok(m) => NextRequest::Got(m.request, m.message_id, m.payload.0),
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
        queue: &QueueManager,
    ) -> Option<Vec<u8>> {
        let reply_rx = match queue.submit(
            msg_id,
            request.clone(),
            payload,
            self.socket_id,
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
                        method: "SocketSession::run_request".to_string(),
                        probable_cause: "the queue executor isn't running".to_string(),
                    },
                );
                let err: Result<(), DbError> =
                    Err(DbError::BadCode(format!("queue closed: {e}")));
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
                        method: "SocketSession::run_request".to_string(),
                        probable_cause: "the executor task ended".to_string(),
                    },
                );
                let err: Result<(), DbError> =
                    Err(DbError::BadCode("queue dropped the reply".to_string()));
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

    async fn send_nudge(&mut self) -> bool {
        let nudge_response = Response {
            message_id: 0,
            request: Request::Nudge,
            payload: Base64Bytes(Vec::new()),
        };

        let json = match response_to_json_str(&nudge_response) {
            Ok(j) => j,
            Err(e) => {
                log_error(
                    &self.label,
                    LogCategory::LetterDelivery,
                    &ErrorDetails {
                        err_msg: format!("nudge: serialize failed: {e}"),
                        file: file!().to_string(),
                        method: "SocketSession::send_nudge".to_string(),
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
                        err_msg: format!("nudge: send failed: {e}"),
                        file: file!().to_string(),
                        method: "SocketSession::send_nudge".to_string(),
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
    let (nudge_tx, nudge_rx) = tokio::sync::mpsc::unbounded_channel();
    telephone.register_socket(socket_id, nudge_tx);

    let mut session = SocketSession::new(socket, socket_id, addr, nudge_rx);
    session.run(&telephone, &queue).await;

    telephone.remove_socket(socket_id);
}