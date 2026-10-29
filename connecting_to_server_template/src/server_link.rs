//! Connection lifecycle state, generation signals, counts, and the boot
//! sequence. Mirrors `cqrs_state_synchronization/web_interface/src/server_link/`.
//! This file starts with state types and helpers only. The receive path,
//! pending queue and boot sequence arrive in later steps.

use leptos::prelude::*;
use std::collections::{HashSet, VecDeque};
use std::sync::LazyLock;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ConnState {
    Offline,
    Connecting,
    FailedToConnect,
    Connected,
}

static CONN_STATE: LazyLock<ArcRwSignal<ConnState>> =
    LazyLock::new(|| ArcRwSignal::new(ConnState::Offline));

static CURRENT_LOGIN_GENERATION: LazyLock<ArcRwSignal<Option<i64>>> =
    LazyLock::new(|| ArcRwSignal::new(None));

static STORED_DATASET_GENERATION: LazyLock<ArcRwSignal<Option<i64>>> =
    LazyLock::new(|| ArcRwSignal::new(None));

static PENDING_COUNT: LazyLock<ArcRwSignal<usize>> =
    LazyLock::new(|| ArcRwSignal::new(0));

static APPLIED_COUNT: LazyLock<ArcRwSignal<usize>> =
    LazyLock::new(|| ArcRwSignal::new(0));

pub const TEMPLATE_USER_NAME: &str = "template_user";

pub enum GenerationCheck {
    FirstTime,
    Same,
    #[allow(dead_code)]
    Changed,
}

pub fn conn_state_signal() -> ArcRwSignal<ConnState> {
    CONN_STATE.clone()
}

pub fn current_generation() -> Option<i64> {
    CURRENT_LOGIN_GENERATION.get_untracked()
}

pub fn current_conn_state() -> ConnState {
    CONN_STATE.get_untracked()
}

pub fn current_restart_in_flight() -> bool {
    RESTART_IN_FLIGHT.get_untracked()
}

/// Lonewolf: socket is up and login succeeded, but sync is gated.
pub fn is_lonewolf() -> bool {
    current_conn_state() == ConnState::Connected && crate::sync_mode::is_gated()
}

// PLACEHOLDER: the real apply path wipes every local page row, then builds
// each server page locally via build_merge_new_page inside one transaction.
// Here it only logs the count.
pub fn apply_server_pages_locally(pages: &[crate::protocol::PageRow]) {
    leptos::logging::log!(
        "apply_server_pages_locally: applying {} pages",
        pages.len()
    );
}

pub enum RestartOutcome {
    Synced,
    AlreadyStopped,
    JustChanged,
    Failed(String),
}

pub fn stored_generation() -> Option<i64> {
    STORED_DATASET_GENERATION.get_untracked()
}

pub fn pending_count_signal() -> ArcRwSignal<usize> {
    PENDING_COUNT.clone()
}

pub fn current_generation_signal() -> ArcRwSignal<Option<i64>> {
    CURRENT_LOGIN_GENERATION.clone()
}

pub fn stored_generation_signal() -> ArcRwSignal<Option<i64>> {
    STORED_DATASET_GENERATION.clone()
}

pub fn applied_count_signal() -> ArcRwSignal<usize> {
    APPLIED_COUNT.clone()
}

pub fn record_new_generation(new_generation: i64) {
    CURRENT_LOGIN_GENERATION.set(Some(new_generation));
    STORED_DATASET_GENERATION.set(Some(new_generation));
}

pub fn require_server() -> Result<i64, String> {
    if CONN_STATE.get_untracked() != ConnState::Connected {
        return Err("server not connected".into());
    }
    match CURRENT_LOGIN_GENERATION.get_untracked() {
        Some(g) => Ok(g),
        None => Err("not logged in yet".into()),
    }
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

// PLACEHOLDER: the real `clear_seen_everywhere` clears the client's
// `seen_server_letters` table and the in-memory seen-ids set. Here it is
// a no-op until the receive path lands.
pub fn clear_seen_everywhere() {
    clear_seen_set();
}

static PENDING_QUEUE: LazyLock<Mutex<VecDeque<crate::protocol::LoveLetter>>> =
    LazyLock::new(|| Mutex::new(VecDeque::new()));

static SENDING: LazyLock<ArcRwSignal<bool>> =
    LazyLock::new(|| ArcRwSignal::new(false));

static LAST_SEND_FAILED: LazyLock<ArcRwSignal<bool>> =
    LazyLock::new(|| ArcRwSignal::new(false));

static RESTART_IN_FLIGHT: LazyLock<ArcRwSignal<bool>> =
    LazyLock::new(|| ArcRwSignal::new(false));

static IGNORED_WHILE_STOPPED: LazyLock<ArcRwSignal<usize>> =
    LazyLock::new(|| ArcRwSignal::new(0));

static CLIENT_STARTED: LazyLock<ArcRwSignal<bool>> =
    LazyLock::new(|| ArcRwSignal::new(false));

static SEEN_LETTERS: LazyLock<Mutex<HashSet<i64>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

pub fn sending_signal() -> ArcRwSignal<bool> {
    SENDING.clone()
}

pub fn last_send_failed_signal() -> ArcRwSignal<bool> {
    LAST_SEND_FAILED.clone()
}

pub fn restart_in_flight_signal() -> ArcRwSignal<bool> {
    RESTART_IN_FLIGHT.clone()
}

pub fn ignored_while_stopped_signal() -> ArcRwSignal<usize> {
    IGNORED_WHILE_STOPPED.clone()
}

pub fn client_started_signal() -> ArcRwSignal<bool> {
    CLIENT_STARTED.clone()
}

pub fn has_seen(letter_id: i64) -> bool {
    SEEN_LETTERS.lock().expect("seen poisoned").contains(&letter_id)
}

pub fn mark_seen(letter_id: i64) {
    SEEN_LETTERS.lock().expect("seen poisoned").insert(letter_id);
}

pub fn clear_seen_set() {
    SEEN_LETTERS.lock().expect("seen poisoned").clear();
}

pub fn queue_pretend_edit(
    coms: &crate::client_server_coms::ComsHandleForClient,
    page_id: impl Into<String>,
    text: impl Into<String>,
) {
    let letter = crate::protocol::LoveLetter::pretend_edit(page_id, text);
    {
        let mut q = PENDING_QUEUE.lock().expect("pending queue poisoned");
        q.push_back(letter);
        PENDING_COUNT.set(q.len());
    }
    kick_send(coms.clone());
}

pub fn clear_pending_letters() {
    let mut q = PENDING_QUEUE.lock().expect("pending queue poisoned");
    q.clear();
    PENDING_COUNT.set(0);
}

pub fn kick_send(coms: crate::client_server_coms::ComsHandleForClient) {
    if crate::sync_mode::is_gated() {
        return;
    }
    if CONN_STATE.get_untracked() != ConnState::Connected {
        return;
    }
    if SENDING.get_untracked() {
        return;
    }
    SENDING.set(true);
    let coms = coms;
    leptos::task::spawn_local(async move {
        loop {
            if crate::sync_mode::is_gated() {
                break;
            }
            if CONN_STATE.get_untracked() != ConnState::Connected {
                break;
            }
            let letter = {
                let q = PENDING_QUEUE.lock().expect("pending queue poisoned");
                q.front().cloned()
            };
            let Some(letter) = letter else { break };
            match coms.send_letter(letter).await {
                Ok(()) => {
                    LAST_SEND_FAILED.set(false);
                    let mut q = PENDING_QUEUE.lock().expect("pending queue poisoned");
                    q.pop_front();
                    PENDING_COUNT.set(q.len());
                }
                Err(e) => {
                    LAST_SEND_FAILED.set(true);
                    if matches!(e, crate::client_server_coms::ComsError::Server(_)) {
                        let n = PENDING_COUNT.get_untracked();
                        crate::cool_popup::show_popup_as(
                            crate::cool_popup::PopupKind::Error,
                            format!("Sending failed: {e}. {n} edits still queued; they retry on the next edit or reconnect."),
                        );
                    }
                    break;
                }
            }
        }
        SENDING.set(false);
    });
}

static BOOTSTRAPPED: AtomicBool = AtomicBool::new(false);

pub async fn restart_sync_stack(
    coms: crate::client_server_coms::ComsHandleForClient,
) -> RestartOutcome {
    RESTART_IN_FLIGHT.set(true);
    let result = run_restart(coms).await;
    RESTART_IN_FLIGHT.set(false);
    result
}

async fn run_restart(
    coms: crate::client_server_coms::ComsHandleForClient,
) -> RestartOutcome {
    CONN_STATE.set(ConnState::Connecting);
    if !coms.is_server_online().await {
        CONN_STATE.set(ConnState::FailedToConnect);
        return RestartOutcome::Failed("the server is offline".into());
    }
    CONN_STATE.set(ConnState::Connected);

    let login = match coms.login(TEMPLATE_USER_NAME).await {
        Ok(p) => p,
        Err(e) => {
            CONN_STATE.set(ConnState::FailedToConnect);
            CURRENT_LOGIN_GENERATION.set(None);
            return RestartOutcome::Failed(format!("login failed: {e}"));
        }
    };
    CURRENT_LOGIN_GENERATION.set(Some(login.dataset_generation));
    let mut just_changed = false;
    match check_login_generation(login.dataset_generation) {
        GenerationCheck::FirstTime => {
            STORED_DATASET_GENERATION.set(Some(login.dataset_generation));
        }
        GenerationCheck::Same => {}
        GenerationCheck::Changed => {
            let was_already = crate::sync_mode::current_mode()
                == crate::sync_mode::SyncMode::NeedsReconcile;
            crate::sync_mode::set_sync_mode(crate::sync_mode::SyncMode::NeedsReconcile);
            crate::sync_mode::set_mode_reason(crate::sync_mode::ModeReason::ServerChanged);
            clear_seen_everywhere();
            just_changed = !was_already;
        }
    }

    if crate::sync_mode::is_gated() {
        return if just_changed {
            RestartOutcome::JustChanged
        } else {
            RestartOutcome::AlreadyStopped
        };
    }
    kick_send(coms.clone());
    if !BOOTSTRAPPED.swap(true, Ordering::AcqRel) {
        if let Err(e) = start_over_fresh(&coms).await {
            return RestartOutcome::Failed(format!("bootstrap failed: {e}"));
        }
    }
    if let Err(e) = pull_unsent(&coms).await {
        return RestartOutcome::Failed(format!("pull failed: {e}"));
    }
    RestartOutcome::Synced
}

pub fn receive_letter(coms: &crate::client_server_coms::ComsHandleForClient, letter: crate::protocol::UnsentLetter) {
    if crate::sync_mode::is_gated() {
        IGNORED_WHILE_STOPPED.update(|n| *n += 1);
        return;
    }
    if has_seen(letter.letter_id) {
        coms.ack(letter.letter_id);
        return;
    }
    // PLACEHOLDER: the real apply path merges the letter's snapshot into
    // the local boss and mirror rows. Here it logs, counts, and acks.
    leptos::logging::log!(
        "receive: applying letter_id={} target={}",
        letter.letter_id,
        match &letter.love_letter {
            crate::protocol::LoveLetter::EditBlock { target_page_id, .. } => target_page_id.clone(),
            crate::protocol::LoveLetter::CreateNewPage { page_id, .. } => page_id.clone(),
            crate::protocol::LoveLetter::DisablePage { page_id, .. } => page_id.clone(),
        }
    );
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

pub async fn start_over_fresh(
    coms: &crate::client_server_coms::ComsHandleForClient,
) -> Result<(), String> {
    let payload = coms
        .get_all_pages()
        .await
        .map_err(|e| e.to_string())?;
    // PLACEHOLDER: the real `start_over_fresh` refuses to run if the local
    // db already has pages, then builds new local pages from every row.
    // Here it only logs the row count.
    leptos::logging::log!("server_link: start_over_fresh got {} pages", payload.pages.len());
    Ok(())
}
