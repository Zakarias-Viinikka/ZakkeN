//! In-memory cache for session id and user id.
//!
//! Same idea as the Android app's FastStorage. Read once from
//! key_value_storage at startup, keep the value in a signal, every
//! later read is a sync call. Writes go to both storage and the
//! signal.
//!
//! Both start as None. `init_all` in `init.rs` fills them before
//! the router mounts. If anything reads them before that, that's a
//! bug in the boot order, not something to recover from.

use std::sync::LazyLock;

use leptos::prelude::*;

pub const INIT_ERR: &str =
    "cached_ids not initialized; init::init_all must run before anything reads them";

pub static SESSION_ID: LazyLock<ArcRwSignal<Option<String>>> =
    LazyLock::new(|| ArcRwSignal::new(None));

pub static USER_ID: LazyLock<ArcRwSignal<Option<String>>> =
    LazyLock::new(|| ArcRwSignal::new(None));

pub static INIT_DONE: LazyLock<ArcRwSignal<bool>> =
    LazyLock::new(|| ArcRwSignal::new(false));
