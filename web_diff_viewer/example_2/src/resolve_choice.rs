//! Caller pov: what the user decided for one mismatched page. `Mine`
//! means the client's version wins. `Server` means the server's version
//! wins (nothing to send). `Merged` carries whatever block list the diff
//! viewer produced when the user clicked Done.

// The `Merged` payload mirrors the real reconcile page, which reads it
// when building letters for the server. Nothing here reads it yet.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ResolveChoice {
    Mine,
    Server,
    Merged(Vec<String>),
}