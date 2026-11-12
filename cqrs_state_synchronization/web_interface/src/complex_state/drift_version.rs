//! The client's current drift version.
//!
//! Two copies exist. The in-memory cache lives in `shared_internals`
//! because the router reads it synchronously on every send, before any
//! async call can run. The DB copy lives in `key_value_storage` because
//! it must survive reloads. This file is where both are kept in sync.
//!
//! `plexget_drift_version()` reads the cache and panics if the cache
//! hasn't been populated yet. The panic is deliberate: the caller's
//! logic promised the cache would be populated, and if it isn't, that
//! logic is wrong. The one caller that can't make that promise is the
//! login comparison itself, which uses `plexget_drift_version_from_db()`.
//!
//! Touchers (literal path from the web_interface `src/` root). Each
//! says what would stop happening if that toucher didn't exist.
//!
//! Getters:
//! - `owner_of_interacting_with_server/main_for_login_and_registration.rs`
//!   — reads the DB value (via `plexget_drift_version_from_db`) for the
//!   login comparison, before the cache is populated.
//! - `shared_internals::get_drift_version()` — reads the cache directly
//!   on every send from the router. Not this file's getter, but the
//!   same cache.
//!
//! Setters:
//! - `owner_of_interacting_with_server/main_for_login_and_registration.rs`
//!   writes both copies after login succeeds. Without it, no request
//!   would ever carry a version and the server would refuse every one.
//! - `db/dev_actions.rs` (the deliberate-lonewolf button) writes both
//!   copies. Without it, the client would enter lonewolf but every send
//!   would still carry the old, valid version.
//! - `db/dev_actions.rs::wipe_local_and_pull_from_server` deletes the
//!   DB copy only. The reload that follows re-populates the cache from
//!   the DB on next login, so the cache is stale for the brief moment
//!   between delete and reload but nothing runs in that window.

use error_stuff::CqrsErr;
use leptos::prelude::GetUntracked;

use crate::db::cached_ids;
use crate::db::key_value_storage_db_actions;

/// Current drift version from the in-memory cache. Panics if the cache
/// hasn't been populated. Callers must know the cache is populated;
/// the only legitimate caller at that point is anything after login
/// has finished.
pub fn plexget_drift_version() -> String {
    shared_internals::get_drift_version().unwrap_or_else(|| {
        panic!(
            "plexget_drift_version called before the cache was populated. \
             Callers before login should use plexget_drift_version_from_db."
        )
    })
}

/// Current drift version from the DB. `None` if never set. Async
/// because it reads from `key_value_storage`. Use this only where the
/// cache can't be trusted yet — i.e. the login comparison.
pub async fn plexget_drift_version_from_db() -> Result<Option<String>, CqrsErr> {
    key_value_storage_db_actions::get_client_drift_version().await
}

/// Write both copies. DB first, then cache — so a DB failure leaves
/// the cache untouched. Use this when the caller has a specific
/// version to set (login adopts the server's).
pub async fn plexset_drift_version(v: String) -> Result<(), CqrsErr> {
    key_value_storage_db_actions::set_client_drift_version(&v).await?;
    shared_internals::set_drift_version(v);
    Ok(())
}

/// Mint a fresh local drift version and write both copies. Use this to
/// deliberately cause a mismatch (the dev-panel lonewolf button).
/// Caller doesn't need to know what the new version is.
pub async fn plexset_make_new_drift_version() -> Result<(), CqrsErr> {
    let user_id = cached_ids::USER_ID
        .get_untracked()
        .unwrap_or_else(|| panic!("plexset_make_new_drift_version: USER_ID not set"));
    let time = crate::get_time();
    let v = drift_guard_shared::new_drift_version(&user_id, &time);
    plexset_drift_version(v).await
}