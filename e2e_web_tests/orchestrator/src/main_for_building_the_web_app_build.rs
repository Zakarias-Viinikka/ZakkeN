//! Runs `trunk build` in web_interface so dist/ exists before the orchestrator
//! starts serving it.

use std::path::PathBuf;

pub fn main_for_building_the_web_app_build() -> Result<(), String> {
    let web_app = web_interface_folder()?;
    let status = std::process::Command::new("trunk")
        .arg("build")
        .current_dir(&web_app)
        .status()
        .map_err(|e| format!("could not run trunk build: {e}"))?;
    if !status.success() {
        return Err(format!("trunk build in web_interface failed: {status}"));
    }
    Ok(())
}

fn web_interface_folder() -> Result<PathBuf, String> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("cqrs_state_synchronization/web_interface"))
        .ok_or_else(|| "orchestrator crate has no grandparent folder".to_string())
}
