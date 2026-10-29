//! Connection lifecycle state, generation signals, counts, and the boot
//! sequence. Mirrors `cqrs_state_synchronization/web_interface/src/server_link/`.
//! This file starts with state types and helpers only. The receive path,
//! pending queue and boot sequence arrive in later steps.

use leptos::prelude::*;
use std::collections::VecDeque;
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
pub fn clear_seen_everywhere() {}

static PENDING_QUEUE: LazyLock<Mutex<VecDeque<crate::protocol::LoveLetter>>> =
    LazyLock::new(|| Mutex::new(VecDeque::new()));
static DRAIN_RUNNING: AtomicBool = AtomicBool::new(false);

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
    if DRAIN_RUNNING.swap(true, Ordering::AcqRel) {
        return;
    }
    let coms = coms;
    leptos::task::spawn_local(async move {
        loop {
            if crate::sync_mode::is_gated() {
                break;
            }
            let letter = {
                let q = PENDING_QUEUE.lock().expect("pending queue poisoned");
                q.front().cloned()
            };
            let Some(letter) = letter else { break };
            match coms.send_letter(letter).await {
                Ok(()) => {
                    let mut q = PENDING_QUEUE.lock().expect("pending queue poisoned");
                    q.pop_front();
                    PENDING_COUNT.set(q.len());
                }
                Err(_) => break,
            }
        }
        DRAIN_RUNNING.store(false, Ordering::Release);
    });
}

static BOOTSTRAPPED: AtomicBool = AtomicBool::new(false);

pub async fn restart_sync_stack(
    coms: crate::client_server_coms::ComsHandleForClient,
) -> Result<(), String> {
    // Connect phase.
    CONN_STATE.set(ConnState::Connecting);
    if !coms.is_server_online().await {
        CONN_STATE.set(ConnState::FailedToConnect);
        return Err("server is offline".into());
    }
    CONN_STATE.set(ConnState::Connected);

    // Login phase.
    let login = match coms.login(TEMPLATE_USER_NAME).await {
        Ok(p) => p,
        Err(e) => {
            crate::cool_popup::show_popup(format!("login failed: {e}"));
            return Err(e.to_string());
        }
    };
    CURRENT_LOGIN_GENERATION.set(Some(login.dataset_generation));
    match check_login_generation(login.dataset_generation) {
        GenerationCheck::FirstTime => {
            STORED_DATASET_GENERATION.set(Some(login.dataset_generation));
        }
        GenerationCheck::Same => {}
        GenerationCheck::Changed => {
            let was_already = crate::sync_mode::current_mode()
                == crate::sync_mode::SyncMode::NeedsReconcile;
            crate::sync_mode::set_sync_mode(crate::sync_mode::SyncMode::NeedsReconcile);
            clear_seen_everywhere();
            if !was_already {
                crate::cool_popup::show_popup(
                    "server dataset changed; sync is paused until Reconcile or Adopt",
                );
            }
        }
    }

    // Gated tail.
    if crate::sync_mode::is_gated() {
        return Ok(());
    }
    kick_send(coms.clone());
    if !BOOTSTRAPPED.swap(true, Ordering::AcqRel) {
        start_over_fresh(&coms).await?;
    }
    pull_unsent(&coms).await?;
    Ok(())
}

pub fn receive_letter(coms: &crate::client_server_coms::ComsHandleForClient, letter: crate::protocol::UnsentLetter) {
    if crate::sync_mode::is_gated() {
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
