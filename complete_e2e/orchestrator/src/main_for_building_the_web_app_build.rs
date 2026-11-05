//! Runs `trunk build` in web_interface when the sources have changed.
//!
//! Freshness is content-based, not mtime-based. mtime comparison was
//! missing changes: coarse timestamps, atomic writes and a second
//! `trunk serve` touching dist can all leave dist newer than a source
//! that has actually changed.
//!
//! A marker file `web_interface/dist/.source_fingerprint` stores a hash
//! of every tracked input after a successful trunk build. Next run
//! recomputes the hash and compares. Different → rebuild. Same →
//! skip. No marker → rebuild.

use std::path::{Path, PathBuf};

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
    mark_build_fresh(&web_app)?;
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
    if !web_app.join("dist/index.html").exists() {
        return Ok(false);
    }
    let marker = web_app.join("dist/.source_fingerprint");
    let stored = std::fs::read_to_string(&marker)
        .ok()
        .map(|s| s.trim().to_string());
    let current = source_fingerprint(web_app)?;
    Ok(stored.as_deref() == Some(current.as_str()))
}

fn mark_build_fresh(web_app: &Path) -> Result<(), String> {
    let marker = web_app.join("dist/.source_fingerprint");
    let current = source_fingerprint(web_app)?;
    std::fs::write(&marker, current).map_err(|e| format!("could not write marker: {e}"))
}

fn source_fingerprint(web_app: &Path) -> Result<String, String> {
    let mut paths = tracked_inputs(web_app);
    paths.sort();
    let mut total: u64 = 0;
    for path in paths {
        if let Ok(bytes) = std::fs::read(&path) {
            for &b in &bytes {
                total = total.wrapping_mul(31).wrapping_add(b as u64);
            }
        }
    }
    Ok(format!("{:x}", total))
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