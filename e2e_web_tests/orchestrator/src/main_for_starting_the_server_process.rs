//! Spawns the zakken_server binary on the requested port, with a fresh temp
//! db, and waits until the port accepts TCP before returning.

use crate::main_for_building_the_server_binary::zakken_server_folder;
use std::time::Duration;

pub struct ServerProcess {
    pub child: tokio::process::Child,
}

pub async fn main_for_starting_the_server_process(port: u16) -> Result<ServerProcess, String> {
    let server_binary = zakken_server_folder()?.join("target/debug/server_core");

    let db_path = "/tmp/zakken_e2e_world.sqlite";
    let _ = std::fs::remove_file(db_path);
    let _ = std::fs::remove_file(format!("{db_path}-wal"));
    let _ = std::fs::remove_file(format!("{db_path}-shm"));

    let child = tokio::process::Command::new(&server_binary)
        .arg("zzport")
        .arg(port.to_string())
        .arg("zzpath")
        .arg(db_path)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("could not spawn {}: {e}", server_binary.display()))?;

    wait_until_server_listens(port, Duration::from_secs(10)).await?;
    Ok(ServerProcess { child })
}

async fn wait_until_server_listens(port: u16, timeout: Duration) -> Result<(), String> {
    let deadline = std::time::Instant::now() + timeout;
    loop {
        if tokio::net::TcpStream::connect(("127.0.0.1", port))
            .await
            .is_ok()
        {
            return Ok(());
        }
        if std::time::Instant::now() >= deadline {
            return Err(format!(
                "server never listened on 127.0.0.1:{port} within {timeout:?}"
            ));
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
