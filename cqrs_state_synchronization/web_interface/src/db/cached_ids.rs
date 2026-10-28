use std::sync::LazyLock;

use error_stuff::CqrsErr;
use leptos::prelude::*;
use protocol::error::DbError;

pub const INIT_ERR: &str =
    "cached_ids not initialized; init::init_all must run before anything reads them";

pub static SESSION_ID: LazyLock<ArcRwSignal<Option<String>>> =
    LazyLock::new(|| ArcRwSignal::new(None));

pub static USER_ID: LazyLock<ArcRwSignal<Option<String>>> =
    LazyLock::new(|| ArcRwSignal::new(None));

pub static INIT_DONE: LazyLock<ArcRwSignal<bool>> =
    LazyLock::new(|| ArcRwSignal::new(false));

pub fn current_session_id() -> String {
    SESSION_ID.get_untracked().expect(INIT_ERR)
}

pub fn current_user_id() -> String {
    USER_ID.get_untracked().expect(INIT_ERR)
}

pub fn try_current_user_id() -> Result<String, CqrsErr> {
    USER_ID.get_untracked().ok_or_else(|| {
        CqrsErr::DbErrorContainer(DbError::IllegalInput("USER_ID not set".to_string()))
    })
}