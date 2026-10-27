use axum::{
    Router,
    extract::{ConnectInfo, State, ws::WebSocketUpgrade},
    response::Response as HttpResponse,
    routing::any,
};
use server_core::telephone::Telephone;
use queue::QueueManager;
use server_core::logging::log;
use server_core::{queue_executor, setup, socket};
use std::net::SocketAddr;
use tokio::sync::mpsc;

#[derive(Clone)]
struct AppState {
    telephone: Telephone,
    queue: QueueManager,
}

async fn forward_app_state_to_new_socket(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(app): State<AppState>,
) -> HttpResponse {
    ws.on_upgrade(move |socket| socket::socket_main(socket, app.telephone, app.queue, addr))
}

#[tokio::main]
async fn main() {
    // figured this should be noted somewhere. when the server sends response
    // spontaneously, the msg_id is 0, that's an expectation/assumption the
    // server and the clients are just going to be making

    //state just holds the db handle as of writing this comment
    let Some(state) = setup::setup_db() else {
        return;
    };

    //the logs are saved in a table and the queue executor "owns" doing
    // anything related to db, so log_tx is used for adding to the "logging queue"
    // which is a separate queue for logs
    let (log_tx, log_rx) = server_core::logging::setup_logging();

    // telephone is how one client can do what essentially resembles talking to
    // other clients, which is necessary because one client creating a new love
    // letter should hand it over to the currently connected ones. it's also how
    // the server nudges clients to request the love letters they haven't received.
    let telephone = Telephone::new(log_tx);

    //rxtx for important queue, and normal queue.
    let (important_tx, important_rx) = mpsc::unbounded_channel();
    let (normal_tx, normal_rx) = mpsc::unbounded_channel();
    let queue = QueueManager::new(important_tx, normal_tx);

    queue_executor::start(important_rx, normal_rx, log_rx, state, telephone.clone());

    let app_state = AppState { telephone, queue };

    let app = Router::new()
        .route("/ws", any(forward_app_state_to_new_socket))
        .with_state(app_state)
        .into_make_service_with_connect_info::<SocketAddr>();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    let local = listener
        .local_addr()
        .map(|a| a.to_string())
        .unwrap_or_else(|_| "0.0.0.0:3000".to_string());
    log("server", format!("listening on {local}"));
    axum::serve(listener, app).await.unwrap();
}
