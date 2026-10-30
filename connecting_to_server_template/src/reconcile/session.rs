use crate::reconcile::plan::ReconcilePlan;
use crate::sync_mode::SyncMode;
use std::sync::{LazyLock, Mutex};

pub struct ReconcileSession {
    pub previous_mode: SyncMode,
    pub previous_reason: Option<crate::sync_mode::ModeReason>,
    pub apply_attempted: bool,
    pub plan: ReconcilePlan,
}

pub static SESSION: LazyLock<Mutex<Option<ReconcileSession>>> =
    LazyLock::new(|| Mutex::new(None));

pub enum RestoreAction {
    NoSession,
    Restore(SyncMode, Option<crate::sync_mode::ModeReason>),
    ApplyHadStarted,
}

pub fn take_session_for_cancel() -> RestoreAction {
    let mut guard = SESSION.lock().expect("session poisoned");
    match guard.take() {
        None => RestoreAction::NoSession,
        Some(s) if s.apply_attempted => RestoreAction::ApplyHadStarted,
        Some(s) => RestoreAction::Restore(s.previous_mode, s.previous_reason),
    }
}
