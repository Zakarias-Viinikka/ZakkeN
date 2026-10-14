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
use server_core::reply::{craft_response, send_message};
use server_core::server_state::ServerState;
use db_core::personal_db_wrapper;

use std::net::SocketAddr;

async fn ws_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<std::sync::Mutex<ServerState>>>,
) -> HttpResponse {
    println!(
        "--- New WebSocket connection attempt detected! | ip: {} ---",
        addr
    );
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<std::sync::Mutex<ServerState>>) {
    let mut push_rx: Option<UnboundedReceiver<i64>> = None;
    let mut registered_as: Option<String> = None;

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
                    println!("got message: {:?}", json_msg);
                    let parsed = json_str_to_message(json_msg);
                    let (msg_request, msg_id, msg_content);
                    match parsed {
                        Ok(m) => {
                            msg_request = m.request;
                            msg_id = m.message_id;
                            msg_content = m.payload.0;
                        }
                        Err(e) => {
                            if let Err(e2) = socket
                                .send(Message::Text("Message doesn't use correct protocol".into()))
                                .await
                            {
                                println!(
                                    "failed to receive message: '{}', with error: {}",
                                    json_msg, e2
                                );
                            }
                            println!(
                                "failed to receive message: '{}', with error: {}",
                                json_msg, e
                            );
                            continue;
                        }
                    };

                    let do_request_result = {
                        let mut state_locked = state.lock().unwrap();
                        let (bytes, just_registered) = allowed_requests::do_request(
                            &msg_request,
                            msg_content,
                            &mut state_locked,
                        );
                        if let Some(ref name) = just_registered {
                            push_rx = Some(state_locked.connected.add_client(name.clone()));
                            registered_as = Some(name.clone());
                        }
                        bytes
                    };

                    let response = craft_response(do_request_result, msg_id, msg_request);
                    println!("about to send message: {:?}", response);
                    if let Err(_e) = send_message(&mut socket, response).await {
                        //println!("{}", e);
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
                        println!("failed to load letter {}: {:?}", letter_id, e);
                        continue;
                    }
                };

                let push_response = Response {
                    message_id: 0,
                    request: Request::SendLoveLetter,
                    payload: Base64Bytes(letter_bytes),
                };

                let json = match response_to_json_str(&push_response) {
                    Ok(j) => j,
                    Err(e) => {
                        println!("failed to serialize push: {:?}", e);
                        continue;
                    }
                };

                if socket.send(Message::Text(json.into())).await.is_err() {
                    break;
                }
            }
        }
    }

    if let Some(name) = registered_as {
        let mut state_locked = state.lock().unwrap();
        state_locked.connected.remove_client(&name);
    }
}

#[tokio::main]
async fn main() {
    let liver_inner = match personal_db_wrapper::new_liver() {
        Ok(l) => l,
        Err(e) => {
            println!("failed to open db: {:?}", e);
            return;
        }
    };
    if let Err(e) = personal_db_wrapper::create_table_if_not_exist(&liver_inner) {
        println!("failed to create table: {:?}", e);
    }
    if let Err(e) = personal_db_wrapper::create_data_if_not_exist(&liver_inner) {
        println!("failed to create data: {:?}", e);
    }
    if let Err(e) = db_core::db::user_registration::create_table_if_not_exist(&liver_inner) {
        println!("failed to create registered_users table: {:?}", e);
    }
    if let Err(e) = db_core::db::operations_love_letters::create_tables_if_not_exist(&liver_inner) {
        println!("failed to create love letter tables: {:?}", e);
    }

    let state = Arc::new(std::sync::Mutex::new(ServerState::new(liver_inner)));

    let app = Router::new()
        .route("/ws", any(ws_handler))
        .with_state(state)
        .into_make_service_with_connect_info::<SocketAddr>();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}