//! Runs `python3 spike.py` in the parent folder, forwarding every
//! argument the orchestrator got. Exits with python's exit code.

use std::path::PathBuf;

pub async fn main_for_running_the_python_spike() {
    let folder = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();

    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut cmd = tokio::process::Command::new("python3");
    cmd.arg("spike.py");
    for a in &args {
        cmd.arg(a);
    }
    cmd.current_dir(&folder);

    let status = cmd.status().await.expect("failed to run python3 spike.py");
    std::process::exit(status.code().unwrap_or(-1));
}
