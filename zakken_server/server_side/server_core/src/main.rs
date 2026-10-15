use axum::{
    Router,
    extract::{
        ConnectInfo, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::Response as HttpResponse,
    routing::any,
};
use protocol::serialization::Base64Bytes;
use server_protocol::{Request, Response, json_str_to_message, response_to_json_str};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedReceiver;
use server_core::allowed_requests;
use server_core::logging::{log, request_name};
use server_core::reply::{craft_response, send_message};
use server_core::server_state::ServerState;
use db_core::personal_db_wrapper;

use std::net::SocketAddr;

async fn ws_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<std::sync::Mutex<ServerState>>>,
) -> HttpResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state, addr))
}

fn backfill_unsent(state: &mut ServerState, user_name: &str) {
    let ids = match db_core::db::operations_love_letters::get_unsent_letter_ids_for_user(
        &state.liver,
        user_name,
    ) {
        Ok(ids) => ids,
        Err(e) => {
            log(user_name, format!("backfill failed to read unsent ids: {e:?}"));
            return;
        }
    };

    if ids.is_empty() {
        return;
    }

    log(
        user_name,
        format!("backfill: {} unsent letter(s)", ids.len()),
    );

    for id in ids {
        match state.connected.send_to(user_name, id) {
            Ok(()) => {
                if let Err(e) = db_core::db::operations_love_letters::delete_unsent_row(
                    &state.liver,
                    user_name,
                    id,
                ) {
                    log(
                        user_name,
                        format!("  backfill: delete unsent row for letter {id} failed: {e:?}"),
                    );
                }
            }
            Err(e) => {
                log(
                    user_name,
                    format!("  backfill: push letter {id} failed: {e:?}"),
                );
            }
        }
    }
}

async fn handle_socket(
    mut socket: WebSocket,
    state: Arc<std::sync::Mutex<ServerState>>,
    addr: SocketAddr,
) {
    let mut label = addr.to_string();
    let mut user_name: Option<String> = None;
    log(&label, "connected");

    let mut push_rx: Option<UnboundedReceiver<i64>> = None;

    loop {
        let next_push = async {
            match push_rx.as_mut() {
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

                    let (bytes, just_identified) = {
                        let mut state_locked = state.lock().unwrap();
                        let result = allowed_requests::do_request(
                            &msg_request,
                            msg_content,
                            &mut state_locked,
                            sender.as_deref(),
                        );
                        if let Some(ref name) = result.1 {
                            push_rx = Some(state_locked.connected.add_client(name.clone()));
                            backfill_unsent(&mut state_locked, name);
                        }
                        result
                    };

                    if let Some(ref name) = just_identified {
                        user_name = Some(name.clone());
                        label = name.clone();
                    }

                    let response = craft_response(bytes, msg_id, msg_request);
                    if let Err(e) = send_message(&mut socket, response).await {
                        log(&label, format!("send response failed: {e}"));
                        break;
                    }
                }
            }

            letter_id = next_push => {
                let Some(letter_id) = letter_id else {
                    continue;
                };

                let letter_bytes = {
                    let state_locked = state.lock().unwrap();
                    db_core::db::operations_love_letters::get_letter(
                        &state_locked.liver,
                        letter_id,
                    )
                };

                let letter_bytes = match letter_bytes {
                    Ok(b) => b,
                    Err(e) => {
                        log(&label, format!("push: load letter {letter_id} failed: {e:?}"));
                        continue;
                    }
                };

                log(&label, format!("push letter {letter_id}"));

                let push_response = Response {
                    message_id: 0,
                    request: Request::SendLoveLetter,
                    payload: Base64Bytes(letter_bytes),
                };

                let json = match response_to_json_str(&push_response) {
                    Ok(j) => j,
                    Err(e) => {
                        log(&label, format!("push: serialize letter {letter_id} failed: {e}"));
                        continue;
                    }
                };

                if let Err(e) = socket.send(Message::Text(json.into())).await {
                    log(&label, format!("push: send letter {letter_id} failed: {e}"));
                    break;
                }
            }
        }
    }

    log(&label, "disconnected");
    if let Some(name) = user_name {
        let mut state_locked = state.lock().unwrap();
        state_locked.connected.remove_client(&name);
    }
}

#[tokio::main]
async fn main() {
    let liver_inner = match personal_db_wrapper::new_liver() {
        Ok(l) => l,
        Err(e) => {
            log("server", format!("failed to open db: {e:?}"));
            return;
        }
    };
    if let Err(e) = personal_db_wrapper::create_table_if_not_exist(&liver_inner) {
        log("server", format!("failed to create socket_testing_table: {e:?}"));
    }
    if let Err(e) = personal_db_wrapper::create_data_if_not_exist(&liver_inner) {
        log("server", format!("failed to create socket_testing_table data: {e:?}"));
    }
    if let Err(e) = db_core::db::user_registration::create_table_if_not_exist(&liver_inner) {
        log("server", format!("failed to create registered_users: {e:?}"));
    }
    if let Err(e) = db_core::db::operations_love_letters::create_tables_if_not_exist(&liver_inner) {
        log("server", format!("failed to create love letter tables: {e:?}"));
    }
    if let Err(e) = db_core::db::operations_pages::create_table_if_not_exist(&liver_inner) {
        log("server", format!("failed to create pages: {e:?}"));
    }

    let state = Arc::new(std::sync::Mutex::new(ServerState::new(liver_inner)));

    let app = Router::new()
        .route("/ws", any(ws_handler))
        .with_state(state)
        .into_make_service_with_connect_info::<SocketAddr>();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    let local = listener
        .local_addr()
        .map(|a| a.to_string())
        .unwrap_or_else(|_| "0.0.0.0:3000".to_string());
    log("server", format!("listening on {local}"));
    axum::serve(listener, app).await.unwrap();
}