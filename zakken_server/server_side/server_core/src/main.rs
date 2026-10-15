use axum::{
    Router,
    extract::{
        ConnectInfo, State,
        ws::{WebSocket, WebSocketUpgrade},
    },
    response::Response as HttpResponse,
    routing::any,
};
use connected_clients::ConnectedClients;
use queue::QueueManager;
use server_core::logging::log;
use server_core::server_state::ServerState;
use server_core::{queue_executor, setup, socket};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

#[derive(Clone)]
struct AppState {
    state: Arc<Mutex<ServerState>>,
    clients: ConnectedClients,
    queue: QueueManager,
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(app): State<AppState>,
) -> HttpResponse {
    ws.on_upgrade(move |socket| socket::handle_socket(socket, app.state, app.clients, app.queue, addr))
}

#[tokio::main]
async fn main() {
    let Some(state) = setup::open_and_prepare() else {
        return;
    };

    let (log_tx, log_rx) = mpsc::unbounded_channel();
    server_core::logging::install_log_channel(log_tx.clone());

    let clients = ConnectedClients::new(log_tx);

    let (important_tx, important_rx) = mpsc::unbounded_channel();
    let (normal_tx, normal_rx) = mpsc::unbounded_channel();
    let queue = QueueManager::new(important_tx, normal_tx);

    queue_executor::start(
        important_rx,
        normal_rx,
        log_rx,
        state.clone(),
        clients.clone(),
    );

    let app_state = AppState {
        state,
        clients,
        queue,
    };

    let app = Router::new()
        .route("/ws", any(ws_handler))
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