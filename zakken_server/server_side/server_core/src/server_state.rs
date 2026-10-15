use db_wrapper::android_mascot::LiveForever;

/// The one thing the server needs behind a lock: the sqlite handle.
/// Connections live in their own crate now, so they don't share this lock.
pub struct ServerState {
    pub liver: LiveForever,
}

impl ServerState {
    pub fn new(liver: LiveForever) -> Self {
        ServerState { liver }
    }
}