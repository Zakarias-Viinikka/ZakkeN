mod main_for_building_the_server_binary;
mod main_for_building_the_web_app_build;
mod main_for_finding_a_free_port;
mod main_for_serving_the_web_app_build;
mod main_for_starting_the_server_process;

use main_for_building_the_server_binary::main_for_building_the_server_binary;
use main_for_building_the_web_app_build::main_for_building_the_web_app_build;
use main_for_finding_a_free_port::main_for_finding_a_free_port;
use main_for_serving_the_web_app_build::main_for_serving_the_web_app_build;
use main_for_starting_the_server_process::main_for_starting_the_server_process;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    main_for_building_the_server_binary()?;
    main_for_building_the_web_app_build()?;

    let server_port: u16 = 3000;
    let mut server = main_for_starting_the_server_process(server_port).await?;
    println!("[orchestrator] server at ws://127.0.0.1:{server_port}/ws");

    let static_port = main_for_finding_a_free_port()?;
    let static_listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{static_port}"))
        .await
        .map_err(|e| format!("could not bind static listener: {e}"))?;
    let static_handle = tokio::spawn(main_for_serving_the_web_app_build(static_listener));

    let web_app_url = format!("http://127.0.0.1:{static_port}/");
    println!("[orchestrator] web app at {web_app_url}");
    let _ = std::process::Command::new("xdg-open").arg(&web_app_url).spawn();

    println!("[orchestrator] running. press ctrl-c to stop.");
    tokio::signal::ctrl_c().await?;
    println!("[orchestrator] shutting down");

    let _ = server.child.kill().await;
    let _ = server.child.wait().await;
    static_handle.abort();
    Ok(())
}
