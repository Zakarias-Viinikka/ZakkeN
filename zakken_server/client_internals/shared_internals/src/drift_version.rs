use std::sync::{Mutex, OnceLock};

static DRIFT_VERSION: OnceLock<Mutex<Option<String>>> = OnceLock::new();

fn slot() -> &'static Mutex<Option<String>> {
    DRIFT_VERSION.get_or_init(|| Mutex::new(None))
}

/// Read the drift version the client is currently on. None means the
/// client hasn't learned it yet, which is only legal on register,
/// login, and GetCurrentDrift - the server refuses anything else.
pub fn get_drift_version() -> Option<String> {
    slot().lock().unwrap().clone()
}

/// Set the drift version the client is currently on. Called after
/// login stores the version, and after a wipe reply stores a new one.
pub fn set_drift_version(version: String) {
    *slot().lock().unwrap() = Some(version);
}
