use leptos::prelude::*;
use std::sync::LazyLock;

static CURRENT_LOGIN_GENERATION: LazyLock<ArcRwSignal<Option<i64>>> =
    LazyLock::new(|| ArcRwSignal::new(None));

static STORED_DATASET_GENERATION: LazyLock<ArcRwSignal<Option<i64>>> =
    LazyLock::new(|| ArcRwSignal::new(None));

pub enum GenerationCheck {
    FirstTime,
    Same,
    #[allow(dead_code)]
    Changed,
}

pub fn current_generation() -> Option<i64> {
    CURRENT_LOGIN_GENERATION.get_untracked()
}

pub fn stored_generation() -> Option<i64> {
    STORED_DATASET_GENERATION.get_untracked()
}

pub fn current_generation_signal() -> ArcRwSignal<Option<i64>> {
    CURRENT_LOGIN_GENERATION.clone()
}

pub fn stored_generation_signal() -> ArcRwSignal<Option<i64>> {
    STORED_DATASET_GENERATION.clone()
}

pub fn generations_differ_signal() -> Signal<bool> {
    let curr = CURRENT_LOGIN_GENERATION.clone();
    let stored = STORED_DATASET_GENERATION.clone();
    Signal::derive(move || {
        let c = curr.get();
        let s = stored.get();
        matches!((c, s), (Some(c), Some(s)) if c != s)
    })
}

pub fn record_new_generation(new_generation: i64) {
    CURRENT_LOGIN_GENERATION.set(Some(new_generation));
    STORED_DATASET_GENERATION.set(Some(new_generation));
}

// PLACEHOLDER: the real check compares the server's dataset generation
// against the stored one. Mismatch is unreachable here (single client) and
// is left in place so the branch exists.
pub fn check_login_generation(server_generation: i64) -> GenerationCheck {
    match STORED_DATASET_GENERATION.get_untracked() {
        None => GenerationCheck::FirstTime,
        Some(stored) if stored == server_generation => GenerationCheck::Same,
        Some(_) => GenerationCheck::Changed,
    }
}
