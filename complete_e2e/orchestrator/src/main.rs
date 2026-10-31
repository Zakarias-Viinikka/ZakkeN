mod main_for_building_the_server_binary;
mod main_for_building_the_web_app_build;
mod main_for_finding_a_free_port;
mod main_for_running_the_tester;
mod main_for_serving_the_web_app_build;
mod main_for_starting_the_server_process;

use main_for_building_the_server_binary::main_for_building_the_server_binary;
use main_for_building_the_web_app_build::main_for_building_the_web_app_build;
use main_for_finding_a_free_port::main_for_finding_a_free_port;
use main_for_running_the_tester::main_for_running_the_tester;
use main_for_serving_the_web_app_build::main_for_serving_the_web_app_build;
use main_for_starting_the_server_process::main_for_starting_the_server_process;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    main_for_building_the_server_binary()?;
    main_for_building_the_web_app_build()?;

    let server_port = main_for_finding_a_free_port()?;
    let mut server = main_for_starting_the_server_process(server_port).await?;
    println!("[orchestrator] server websocket at ws://127.0.0.1:{server_port}/ws");

    let static_port = main_for_finding_a_free_port()?;
    let static_listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{static_port}"))
        .await
        .map_err(|e| format!("could not bind static listener: {e}"))?;
    let static_handle = tokio::spawn(main_for_serving_the_web_app_build(static_listener));

    let web_app_url = format!("http://127.0.0.1:{static_port}/?server_port={server_port}");
    println!("[orchestrator] web app at {web_app_url}");

    let exit_code = main_for_running_the_tester(web_app_url).await?;
    println!("[orchestrator] tester finished with exit code {exit_code}");

    let _ = server.child.kill().await;
    let _ = server.child.wait().await;
    static_handle.abort();
    std::process::exit(exit_code);
}