use axum::{
    Router,
    extract::{
        ConnectInfo, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::Response,
    routing::any,
};
use db_wrapper::android_mascot::LiveForever;
use protocol::messages;
use protocol::serialization::json_str_to_message;
use std::sync::Arc;
use zakken_server::reply::{craft_response, send_message};
use zakken_server::{allowed_requests, personal_db_wrapper};

use std::net::SocketAddr;

async fn ws_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(liver): State<Arc<std::sync::Mutex<LiveForever>>>,
) -> Response {
    println!(
        "--- New WebSocket connection attempt detected! | ip: {} ---",
        addr
    );
    ws.on_upgrade(move |socket| handle_socket(socket, liver))
}

async fn handle_socket(mut socket: WebSocket, liver: Arc<std::sync::Mutex<LiveForever>>) {
    while let Some(Ok(msg)) = socket.recv().await {
        if let Message::Text(text) = msg {
            'block: {
                let json_msg = text.as_str();
                println!("got message: {:?}", json_msg);
                let msg = json_str_to_message(json_msg);
                let (msg_request, msg_id, msg_content);
                let msg = match msg {
                    Ok(msg) => {
                        msg_request = msg.request;
                        msg_id = msg.message_id;
                        msg_content = msg.payload.0;
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

                        break 'block;
                    }
                };
                //println!("extracted data out of message into vecu8 and about to use it");
                let do_request_result = {
                    //println!("about to lock");
                    let liver_locked = liver.lock().unwrap();
                    allowed_requests::do_request(&msg_request, msg_content, &liver_locked)
                };
                //println!("lock over and request was fulfilled");
                let response = craft_response(do_request_result, msg_id, msg_request);
                println!("about to send message: {:?}", response);
                let result = send_message(&mut socket, response).await;
                //println!("message was sent");
                if let Err(e) = result {
                    //println!("{}", e);
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let liver_inner = personal_db_wrapper::new_liver();
    if let Err(e) = personal_db_wrapper::create_table_if_not_exist(&liver_inner) {
        println!("failed to create table: {:?}", e);
    }
    if let Err(e) = personal_db_wrapper::create_data_if_not_exist(&liver_inner) {
        println!("failed to create data: {:?}", e);
    }
    let liver = Arc::new(std::sync::Mutex::new(liver_inner));

    //passes liver to ws_handler which passes it to handle_socket
    let app = Router::new()
        .route("/ws", any(ws_handler))
        .with_state(liver)
        .into_make_service_with_connect_info::<SocketAddr>();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
