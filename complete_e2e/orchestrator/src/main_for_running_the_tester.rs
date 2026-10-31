//! Runs pytest in `tester/` against the currently running web app and
//! server. Returns pytest's exit code so the orchestrator can exit with it.
//!
//! The web app URL is handed over as an environment variable; pytest
//! inherits the caller's PATH, so an activated venv wins.

use std::path::PathBuf;

pub async fn main_for_running_the_tester(web_app_url: String) -> Result<i32, String> {
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("orchestrator crate has no parent folder")?
        .to_path_buf();
    let tester = workspace.join("tester");

    let status = tokio::process::Command::new("python3")
        .args(["-m", "pytest", "-v"])
        .current_dir(&tester)
        .env("WEB_APP_URL", &web_app_url)
        .status()
        .await
        .map_err(|e| format!("could not run pytest: {e}"))?;

    Ok(status.code().unwrap_or(-1))
}
