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

    let port = read_port_from_args();
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await.unwrap();
    let local = listener
        .local_addr()
        .map(|a| a.to_string())
        .unwrap_or_else(|_| format!("0.0.0.0:{port}"));
    log("server", format!("listening on {local}"));
    axum::serve(listener, app).await.unwrap();
}

fn read_port_from_args() -> u16 {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(position) = args.iter().position(|a| a == "zzport") else {
        return 3000;
    };
    let Some(value) = args.get(position + 1) else {
        panic!("zzport needs a value");
    };
    match value.parse::<u16>() {
        Ok(port) => port,
        Err(e) => panic!("zzport value '{value}' is not a valid port: {e}"),
    }
}
