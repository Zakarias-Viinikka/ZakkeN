//! Serves web_interface/dist over HTTP. Missing paths without a file
//! extension fall back to index.html, so client-side routes work.

use crate::main_for_building_the_web_app_build::web_interface_folder;
use tokio::net::TcpListener;
use tower_http::services::{ServeDir, ServeFile};

pub async fn main_for_serving_the_web_app_build(listener: TcpListener) -> Result<(), String> {
    let dist = web_interface_folder()?.join("dist");
    let index = dist.join("index.html");

    let app = axum::Router::new()
        .fallback_service(ServeDir::new(&dist).fallback(ServeFile::new(&index)));

    axum::serve(listener, app)
        .await
        .map_err(|e| format!("static server failed: {e}"))
}