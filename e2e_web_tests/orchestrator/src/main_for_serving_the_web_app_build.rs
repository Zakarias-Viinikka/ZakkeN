//! Serves web_interface/dist over HTTP. Missing paths without a file
//! extension fall back to index.html, so client-side routes work.

use std::path::PathBuf;
use tokio::net::TcpListener;
use tower_http::services::{ServeDir, ServeFile};

pub async fn main_for_serving_the_web_app_build(listener: TcpListener) -> Result<(), String> {
    let dist = web_interface_dist_folder()?;
    let index = dist.join("index.html");

    let app = axum::Router::new()
        .fallback_service(ServeDir::new(&dist).fallback(ServeFile::new(&index)));

    axum::serve(listener, app)
        .await
        .map_err(|e| format!("static server failed: {e}"))
}

fn web_interface_dist_folder() -> Result<PathBuf, String> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("cqrs_state_synchronization/web_interface/dist"))
        .ok_or_else(|| "orchestrator crate has no grandparent folder".to_string())
}
