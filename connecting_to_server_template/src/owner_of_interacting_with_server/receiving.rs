use leptos::prelude::*;
use std::collections::HashSet;
use std::sync::LazyLock;
use std::sync::Mutex;

pub fn receive_letter(
    coms: &crate::client_server_coms::ComsHandleForClient,
    letter: crate::protocol::UnsentLetter,
) {
    if crate::owner_of_interacting_with_server::locked::is_locked() {
        IGNORED_WHILE_STOPPED.update(|n| *n += 1);
        return;
    }
    if crate::sync_mode::is_gated() {
        IGNORED_WHILE_STOPPED.update(|n| *n += 1);
        return;
    }
    if has_seen(letter.letter_id) {
        coms.ack(letter.letter_id);
        return;
    }
    apply_letter_locally(&letter);
    mark_seen(letter.letter_id);
    APPLIED_COUNT.update(|n| *n += 1);
    coms.ack(letter.letter_id);
}

pub async fn pull_unsent(
    coms: &crate::client_server_coms::ComsHandleForClient,
) -> Result<(), String> {
    let letters = coms
        .get_my_unsent_letters()
        .await
        .map_err(|e| e.to_string())?;
    for letter in letters {
        receive_letter(coms, letter);
    }
    Ok(())
}

pub fn ignored_while_stopped_signal() -> ArcRwSignal<usize> {
    IGNORED_WHILE_STOPPED.clone()
}

pub fn applied_count_signal() -> ArcRwSignal<usize> {
    APPLIED_COUNT.clone()
}

pub fn clear_seen_everywhere() {
    clear_seen_set();
}

fn has_seen(letter_id: i64) -> bool {
    SEEN_LETTERS.lock().expect("seen poisoned").contains(&letter_id)
}

fn mark_seen(letter_id: i64) {
    SEEN_LETTERS.lock().expect("seen poisoned").insert(letter_id);
}

fn clear_seen_set() {
    SEEN_LETTERS.lock().expect("seen poisoned").clear();
}

fn apply_letter_locally(letter: &crate::protocol::UnsentLetter) {
    // PLACEHOLDER: the real apply path merges the letter's snapshot into
    // the local boss and mirror rows. Here it logs, counts, and acks.
    leptos::logging::log!(
        "receive: applying letter_id={} target={}",
        letter.letter_id,
        letter.love_letter.target_page_id(),
    );
}

static SEEN_LETTERS: LazyLock<Mutex<HashSet<i64>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

static IGNORED_WHILE_STOPPED: LazyLock<ArcRwSignal<usize>> =
    LazyLock::new(|| ArcRwSignal::new(0));

static APPLIED_COUNT: LazyLock<ArcRwSignal<usize>> =
    LazyLock::new(|| ArcRwSignal::new(0));