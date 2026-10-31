use leptos::prelude::*;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::owner_of_interacting_with_server::conn_state::ConnState;
use crate::owner_of_interacting_with_server::dataset_generation::{check_login_generation, GenerationCheck};

static RESTART_IN_FLIGHT: LazyLock<ArcRwSignal<bool>> =
    LazyLock::new(|| ArcRwSignal::new(false));

pub const TEMPLATE_USER_NAME: &str = "template_user";

static BOOTSTRAPPED: AtomicBool = AtomicBool::new(false);

pub enum RestartOutcome {
    Synced,
    AlreadyStopped,
    JustChanged,
    Failed(String),
}

pub fn restart_in_flight_signal() -> ArcRwSignal<bool> {
    RESTART_IN_FLIGHT.clone()
}

pub fn current_restart_in_flight() -> bool {
    RESTART_IN_FLIGHT.get_untracked()
}

pub async fn restart_sync_stack(
    coms: crate::client_server_coms::ComsHandleForClient,
) -> RestartOutcome {
    RESTART_IN_FLIGHT.set(true);
    let result = run_restart(coms).await;
    RESTART_IN_FLIGHT.set(false);
    result
}

async fn reach_server(
    coms: &crate::client_server_coms::ComsHandleForClient,
) -> Result<(), String> {
    crate::owner_of_interacting_with_server::conn_state::conn_state_signal().set(ConnState::Connecting);
    if !coms.is_server_online().await {
        crate::owner_of_interacting_with_server::conn_state::conn_state_signal().set(ConnState::FailedToConnect);
        return Err("the server is offline".into());
    }
    crate::owner_of_interacting_with_server::conn_state::conn_state_signal().set(ConnState::Connected);
    Ok(())
}

async fn log_in(
    coms: &crate::client_server_coms::ComsHandleForClient,
) -> Result<i64, String> {
    // First connect registers; every later connect logs in. The server
    // refuses a duplicate name with ERR_USER_ALREADY_EXISTS, which is the
    // signal to fall through to login. This mirrors the real client, where
    // the first connect registers and later connects log in.
    match coms.register(TEMPLATE_USER_NAME).await {
        Ok(()) => {}
        Err(crate::client_server_coms::ComsError::Server(msg))
            if msg == crate::protocol::ERR_USER_ALREADY_EXISTS => {}
        Err(e) => {
            crate::owner_of_interacting_with_server::conn_state::conn_state_signal().set(ConnState::FailedToConnect);
            crate::owner_of_interacting_with_server::dataset_generation::current_generation_signal().set(None);
            return Err(format!("register failed: {e}"));
        }
    }
    let login = match coms.login(TEMPLATE_USER_NAME).await {
        Ok(p) => p,
        Err(e) => {
            crate::owner_of_interacting_with_server::conn_state::conn_state_signal().set(ConnState::FailedToConnect);
            crate::owner_of_interacting_with_server::dataset_generation::current_generation_signal().set(None);
            return Err(format!("login failed: {e}"));
        }
    };
    crate::owner_of_interacting_with_server::dataset_generation::current_generation_signal()
        .set(Some(login.dataset_generation));
    Ok(login.dataset_generation)
}

fn stop_sync_if_dataset_changed(generation: i64) -> bool {
    let mut just_changed = false;
    match check_login_generation(generation) {
        GenerationCheck::FirstTime => {
            crate::owner_of_interacting_with_server::dataset_generation::stored_generation_signal()
                .set(Some(generation));
        }
        GenerationCheck::Same => {}
        GenerationCheck::Changed => {
            let was_already = crate::sync_mode::current_mode()
                == crate::sync_mode::SyncMode::NeedsReconcile;
            crate::sync_mode::set_sync_mode(crate::sync_mode::SyncMode::NeedsReconcile);
            crate::sync_mode::set_mode_reason(crate::sync_mode::ModeReason::ServerChanged);
            crate::owner_of_interacting_with_server::receiving::clear_seen_everywhere();
            just_changed = !was_already;
        }
    }
    just_changed
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
    leptos::logging::log!(
        "owner_of_interacting_with_server: start_over_fresh got {} pages",
        payload.pages.len()
    );
    Ok(())
}

async fn start_over_fresh_on_first_connect(
    coms: &crate::client_server_coms::ComsHandleForClient,
) -> Result<(), String> {
    if !BOOTSTRAPPED.swap(true, Ordering::AcqRel) {
        return start_over_fresh(coms).await;
    }
    Ok(())
}

async fn run_restart(
    coms: crate::client_server_coms::ComsHandleForClient,
) -> RestartOutcome {
    if crate::owner_of_interacting_with_server::locked::is_locked() {
        return RestartOutcome::Failed("sync locked until restart".into());
    }
    if let Err(e) = reach_server(&coms).await {
        return RestartOutcome::Failed(e);
    }
    let generation = match log_in(&coms).await {
        Ok(g) => g,
        Err(e) => return RestartOutcome::Failed(e),
    };
    let just_changed = stop_sync_if_dataset_changed(generation);

    if crate::sync_mode::is_gated() {
        return if just_changed {
            RestartOutcome::JustChanged
        } else {
            RestartOutcome::AlreadyStopped
        };
    }
    crate::owner_of_interacting_with_server::sending::main_send_unsent_love_letters(coms.clone());
    if crate::owner_of_interacting_with_server::locked::is_locked() {
        return RestartOutcome::Failed("sync locked mid-restart".into());
    }
    if let Err(e) = start_over_fresh_on_first_connect(&coms).await {
        return RestartOutcome::Failed(format!("bootstrap failed: {e}"));
    }
    if let Err(e) = crate::owner_of_interacting_with_server::receiving::pull_unsent(&coms).await {
        return RestartOutcome::Failed(format!("pull failed: {e}"));
    }
    RestartOutcome::Synced
}

// After the server tells us we're lonewolf (another client wiped or
// started over from its own version), our registered user on the server
// is gone. The socket is still up, so this isn't an offline reconnect —
// it's just re-registration. Run `restart_sync_stack` a couple of times
// so we get registered again. Each call does register+login; on success
// we're still NeedsReconcile so we stay lonewolf, waiting for the user
// to pick reconcile or adopt. Silent on success; the message handler
// already popped a warning.
static LONEWOLF_RETRY_RUNNING: LazyLock<ArcRwSignal<bool>> =
    LazyLock::new(|| ArcRwSignal::new(false));

pub fn kick_lonewolf_retry(coms: crate::client_server_coms::ComsHandleForClient) {
    if LONEWOLF_RETRY_RUNNING.get_untracked() {
        return;
    }
    if current_restart_in_flight() {
        // A reconnect is already mid-flight; it does the same work.
        return;
    }
    LONEWOLF_RETRY_RUNNING.set(true);
    leptos::task::spawn_local(async move {
        for _attempt in 0..3 {
            {
                let (tx, rx) = futures::channel::oneshot::channel::<()>();
                leptos::prelude::set_timeout(
                    move || {
                        let _ = tx.send(());
                    },
                    std::time::Duration::from_millis(2000),
                );
                let _ = rx.await;
            }
            if !crate::client::client_started_signal().get_untracked() {
                break;
            }
            let outcome = restart_sync_stack(coms.clone()).await;
            if !matches!(outcome, RestartOutcome::Failed(_)) {
                break;
            }
        }
        LONEWOLF_RETRY_RUNNING.set(false);
    });
}
