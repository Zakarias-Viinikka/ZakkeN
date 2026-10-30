use leptos::prelude::*;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::server_link::conn_state::ConnState;
use crate::server_link::dataset_generation::{check_login_generation, GenerationCheck};

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
    crate::server_link::conn_state::conn_state_signal().set(ConnState::Connecting);
    if !coms.is_server_online().await {
        crate::server_link::conn_state::conn_state_signal().set(ConnState::FailedToConnect);
        return Err("the server is offline".into());
    }
    crate::server_link::conn_state::conn_state_signal().set(ConnState::Connected);
    Ok(())
}

async fn log_in(
    coms: &crate::client_server_coms::ComsHandleForClient,
) -> Result<i64, String> {
    let login = match coms.login(TEMPLATE_USER_NAME).await {
        Ok(p) => p,
        Err(e) => {
            crate::server_link::conn_state::conn_state_signal().set(ConnState::FailedToConnect);
            crate::server_link::dataset_generation::current_generation_signal().set(None);
            return Err(format!("login failed: {e}"));
        }
    };
    crate::server_link::dataset_generation::current_generation_signal()
        .set(Some(login.dataset_generation));
    Ok(login.dataset_generation)
}

fn stop_sync_if_dataset_changed(generation: i64) -> bool {
    let mut just_changed = false;
    match check_login_generation(generation) {
        GenerationCheck::FirstTime => {
            crate::server_link::dataset_generation::stored_generation_signal()
                .set(Some(generation));
        }
        GenerationCheck::Same => {}
        GenerationCheck::Changed => {
            let was_already = crate::sync_mode::current_mode()
                == crate::sync_mode::SyncMode::NeedsReconcile;
            crate::sync_mode::set_sync_mode(crate::sync_mode::SyncMode::NeedsReconcile);
            crate::sync_mode::set_mode_reason(crate::sync_mode::ModeReason::ServerChanged);
            crate::server_link::receiving::clear_seen_everywhere();
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
    leptos::logging::log!("server_link: start_over_fresh got {} pages", payload.pages.len());
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
    crate::server_link::sending::kick_send(coms.clone());
    if let Err(e) = start_over_fresh_on_first_connect(&coms).await {
        return RestartOutcome::Failed(format!("bootstrap failed: {e}"));
    }
    if let Err(e) = crate::server_link::receiving::pull_unsent(&coms).await {
        return RestartOutcome::Failed(format!("pull failed: {e}"));
    }
    RestartOutcome::Synced
}
