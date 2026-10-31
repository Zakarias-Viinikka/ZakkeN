//! Compiles the zakken_server binary so the orchestrator can spawn it.

use std::path::PathBuf;

pub fn main_for_building_the_server_binary() -> Result<(), String> {
    let zakken_server = zakken_server_folder()?;
    let status = std::process::Command::new("cargo")
        .args(["build", "-p", "server_core"])
        .current_dir(&zakken_server)
        .status()
        .map_err(|e| format!("could not run cargo build: {e}"))?;
    if !status.success() {
        return Err(format!("cargo build -p server_core failed: {status}"));
    }
    Ok(())
}

pub fn zakken_server_folder() -> Result<PathBuf, String> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("zakken_server"))
        .ok_or_else(|| "orchestrator crate has no grandparent folder".to_string())
}
