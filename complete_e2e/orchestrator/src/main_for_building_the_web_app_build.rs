//! Runs `trunk build` in web_interface when the build is stale.
//!
//! Cargo is smart enough to skip the server build when nothing changed;
//! trunk is not. It re-runs wasm-bindgen on every call even when no source
//! changed, so we do the freshness check ourselves: `dist/index.html` must
//! exist and be newer than every tracked input.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub fn main_for_building_the_web_app_build() -> Result<(), String> {
    let web_app = web_interface_folder()?;

    if the_existing_build_is_still_fresh(&web_app)? {
        println!("[orchestrator] web_interface build is up to date");
        return Ok(());
    }

    let status = std::process::Command::new("trunk")
        .arg("build")
        .current_dir(&web_app)
        .status()
        .map_err(|e| format!("could not run trunk: {e}"))?;
    if !status.success() {
        return Err(format!("trunk build failed: {status}"));
    }
    Ok(())
}

pub fn web_interface_folder() -> Result<PathBuf, String> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("cqrs_state_synchronization/web_interface"))
        .ok_or_else(|| "orchestrator crate has no grandparent folder".to_string())
}

fn the_existing_build_is_still_fresh(web_app: &Path) -> Result<bool, String> {
    let built_index = web_app.join("dist/index.html");
    let Ok(built_at) = std::fs::metadata(&built_index).and_then(|m| m.modified()) else {
        return Ok(false);
    };

    let mut newest_input: Option<SystemTime> = None;
    for input in tracked_inputs(web_app) {
        let Ok(modified) = std::fs::metadata(&input).and_then(|m| m.modified()) else {
            continue;
        };
        if newest_input.map_or(true, |t| modified > t) {
            newest_input = Some(modified);
        }
    }
    Ok(match newest_input {
        Some(t) => t <= built_at,
        None => true,
    })
}

fn tracked_inputs(web_app: &Path) -> Vec<PathBuf> {
    let mut paths = vec![
        web_app.join("index.html"),
        web_app.join("Cargo.toml"),
        web_app.join("Trunk.toml"),
        web_app.join("trunk.toml"),
        web_app.join("rust-toolchain.toml"),
    ];
    for folder in ["src", "css"] {
        collect_files_recursive(&web_app.join(folder), &mut paths);
    }
    paths
}

fn collect_files_recursive(folder: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files_recursive(&path, out);
        } else {
            out.push(path);
        }
    }
}