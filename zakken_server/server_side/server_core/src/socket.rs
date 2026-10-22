use crate::logging::{log, log_error, request_name};
use crate::reply::{craft_response, send_message};
use axum::extract::ws::{Message, WebSocket};
use connected_clients::Telephone;
use error_stuff::{ErrorDetails, LogCategory};
use protocol::error::DbError;
use protocol::serialization::{Base64Bytes, Convert};
use queue::{Priority, QueueManager};
use server_protocol::{Request, Response, json_str_to_message, response_to_json_str};
use std::net::SocketAddr;
use tokio::sync::mpsc::UnboundedReceiver;

pub async fn handle_socket(
    mut socket: WebSocket,
    telephone: Telephone,
    queue: QueueManager,
    addr: SocketAddr,
) {
    let mut label = addr.to_string();
    let mut user_name: Option<String> = None;
    log(&label, "connected");

    let mut incoming_nudges: Option<UnboundedReceiver<()>> = None;

    loop {
        let next_push = async {
            match incoming_nudges.as_mut() {
                Some(rx) => rx.recv().await,
                None => std::future::pending().await,
            }
        };

        tokio::select! {
            ws_msg = socket.recv() => {
                let Some(Ok(msg)) = ws_msg else {
                    break;
                };

                if let Message::Text(text) = msg {
                    let json_msg = text.as_str();
                    let parsed = json_str_to_message(json_msg);
                    let (msg_request, msg_id, msg_content);
                    match parsed {
                        Ok(m) => {
                            msg_request = m.request;
                            msg_id = m.message_id;
                            msg_content = m.payload.0;
                        }
                        Err(e) => {
                            log(&label, format!("bad message: {e}"));
                            if let Err(e2) = socket
                                .send(Message::Text("Message doesn't use correct protocol".into()))
                                .await
                            {
                                log(&label, format!("send protocol error failed: {e2}"));
                            }
                            continue;
                        }
                    };

                    let sender = user_name.clone();

                    // Log the request first, so the handler's detail lines read
                    // as belonging to it.
                    log(&label, request_name(&msg_request));

                    let request_for_response = msg_request.clone();

                    let reply_rx = match queue.submit(
                        msg_id,
                        msg_request,
                        msg_content,
                        sender,
                        Priority::Normal,
                    ) {
                        Ok(rx) => rx,
                        Err(e) => {
                            log_error(
                                &label,
                                LogCategory::Queue,
                                &ErrorDetails {
                                    err_msg: format!("couldn't queue the request: {e}"),
                                    file: file!().to_string(),
                                    method: "handle_socket".to_string(),
                                    probable_cause: "the queue executor isn't running".to_string(),
                                },
                            );
                            let err: Result<(), DbError> =
                                Err(DbError::BadCode(format!("queue closed: {e}")));
                            let response = craft_response(err.to_payload(), msg_id, request_for_response);
                            if let Err(e2) = send_message(&mut socket, response).await {
                                log(&label, format!("send error failed: {e2}"));
                                break;
                            }
                            continue;
                        }
                    };

                    let reply = match reply_rx.await {
                        Ok(r) => r,
                        Err(_) => {
                            log_error(
                                &label,
                                LogCategory::Queue,
                                &ErrorDetails {
                                    err_msg: "the queue dropped the reply".to_string(),
                                    file: file!().to_string(),
                                    method: "handle_socket".to_string(),
                                    probable_cause: "the executor task ended".to_string(),
                                },
                            );
                            let err: Result<(), DbError> =
                                Err(DbError::BadCode("queue dropped the reply".to_string()));
                            let response = craft_response(err.to_payload(), msg_id, request_for_response);
                            if let Err(e2) = send_message(&mut socket, response).await {
                                log(&label, format!("send error failed: {e2}"));
                                break;
                            }
                            continue;
                        }
                    };

                    if reply.incoming_letters.is_some() {
                        incoming_nudges = reply.incoming_letters;
                    }

                    if let Some(name) = reply.just_identified {
                        user_name = Some(name.clone());
                        label = name;
                    }

                    let response = craft_response(reply.bytes, msg_id, request_for_response);
                    if let Err(e) = send_message(&mut socket, response).await {
                        log(&label, format!("send response failed: {e}"));
                        break;
                    }
                }
            }

            nudge = next_push => {
                let Some(()) = nudge else {
                    // The telephone dropped our sender (another socket for
                    // this user replaced us, or the task ended). Stop
                    // watching for nudges; the rest of the socket keeps
                    // working.
                    incoming_nudges = None;
                    continue;
                };

                let nudge_response = Response {
                    message_id: 0,
                    request: Request::Nudge,
                    payload: Base64Bytes(Vec::new()),
                };

                let json = match response_to_json_str(&nudge_response) {
                    Ok(j) => j,
                    Err(e) => {
                        log_error(
                            &label,
                            LogCategory::LetterDelivery,
                            &ErrorDetails {
                                err_msg: format!("nudge: serialize failed: {e}"),
                                file: file!().to_string(),
                                method: "handle_socket (nudge arm)".to_string(),
                                probable_cause: String::new(),
                            },
                        );
                        continue;
                    }
                };

                if let Err(e) = socket.send(Message::Text(json.into())).await {
                    log_error(
                        &label,
                        LogCategory::LetterDelivery,
                        &ErrorDetails {
                            err_msg: format!("nudge: send failed: {e}"),
                            file: file!().to_string(),
                            method: "handle_socket (nudge arm)".to_string(),
                            probable_cause: "the client's socket closed".to_string(),
                        },
                    );
                    break;
                }
            }
        }
    }

    log(&label, "disconnected");
    if let Some(name) = user_name {
        telephone.remove_client(name);
    }
}
