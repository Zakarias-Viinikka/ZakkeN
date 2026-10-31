//! Spawns the real zakken_server binary on the requested port with a fresh
//! temp db, and waits until the port accepts TCP before returning.
//!
//! Assumes the binary name is `server_core` (the package name in
//! zakken_server/server_side/server_core). If it isn't, both this file and
//! main_for_building_the_server_binary need updating.

use crate::main_for_building_the_server_binary::zakken_server_folder;
use std::path::PathBuf;
use std::time::Duration;

const E2E_DB_PATH: &str = "/tmp/zakken_e2e_world.sqlite";

pub struct ServerProcess {
    pub child: tokio::process::Child,
}

pub async fn main_for_starting_the_server_process(port: u16) -> Result<ServerProcess, String> {
    let server_binary = server_binary_path()?;

    wipe_previous_test_database();

    let child = tokio::process::Command::new(&server_binary)
        .arg("zzport")
        .arg(port.to_string())
        .arg("zzpath")
        .arg(E2E_DB_PATH)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("could not spawn {}: {e}", server_binary.display()))?;

    wait_until_port_answers(port, Duration::from_secs(10)).await?;
    Ok(ServerProcess { child })
}

fn server_binary_path() -> Result<PathBuf, String> {
    Ok(zakken_server_folder()?.join("target/debug/zakken_server"))
}

fn wipe_previous_test_database() {
    let _ = std::fs::remove_file(E2E_DB_PATH);
    let _ = std::fs::remove_file(format!("{E2E_DB_PATH}-wal"));
    let _ = std::fs::remove_file(format!("{E2E_DB_PATH}-shm"));
}

async fn wait_until_port_answers(port: u16, timeout: Duration) -> Result<(), String> {
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
                "server did not answer on 127.0.0.1:{port} within {timeout:?}"
            ));
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}