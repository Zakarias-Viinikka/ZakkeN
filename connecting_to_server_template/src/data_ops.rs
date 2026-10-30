//! Client-side verbs for general data management operations. Each verb
//! reads and writes the sync-mode signal and, when it needs the server,
//! goes through the coms handle. Mirrors the real UI contract in
//! `plan_for_general_data_management_ops.md`.

pub mod busy;
pub mod verb_error;

use crate::client_server_coms::ComsHandleForClient;
use crate::sync_mode::{self, SyncMode};
use leptos::prelude::*;
use busy::with_busy;
use verb_error::VerbError;

pub fn pause() -> Result<(), String> {
    if sync_mode::current_mode() == SyncMode::NeedsReconcile {
        return Err("Sync is already stopped. Reconcile or Adopt.".into());
    }
    sync_mode::set_sync_mode(SyncMode::NeedsReconcile);
    sync_mode::set_mode_reason(sync_mode::ModeReason::YouPaused);
    crate::cool_popup::show_popup_as(
        crate::cool_popup::PopupKind::Warning,
        "Paused. Sync will not resume until you Reconcile or Adopt.",
    );
    Ok(())
}

pub async fn wipe_local_and_pull_from_server(
    coms: ComsHandleForClient,
) -> Result<(), VerbError> {
    with_busy("Wiping local & pulling from server", async move {
        if crate::server_link::conn_state::current_conn_state()
            != crate::server_link::conn_state::ConnState::Connected
        {
            return Err(VerbError::refused(
                "Can't wipe local: not connected to the server",
            ));
        }
        let queued_before = crate::server_link::pending_queue::pending_count_signal().get_untracked();
        let payload = coms
            .get_all_pages()
            .await
            .map_err(|e| VerbError::failed(e.to_string()))?;
        let page_count = payload.pages.len();
        crate::server_link::pending_queue::clear_pending_letters();
        crate::server_link::receiving::clear_seen_everywhere();
        crate::local_pages::apply_server_pages_locally(&payload.pages);
        if let Some(curr) = crate::server_link::dataset_generation::current_generation() {
            crate::server_link::dataset_generation::record_new_generation(curr);
        }
        crate::sync_mode::set_sync_mode(SyncMode::Connected);
        let restart = crate::server_link::restart::restart_sync_stack(coms).await;
        crate::cool_popup::show_popup_as(
            crate::cool_popup::PopupKind::Success,
            format!(
                "Wiped local. Pulled {} pages from server. {} queued edits discarded. Sync resumed.",
                page_count, queued_before,
            ),
        );
        if let crate::server_link::restart::RestartOutcome::Failed(e) = restart {
            crate::cool_popup::show_popup_as(
                crate::cool_popup::PopupKind::Warning,
                format!("Wiped local, but couldn't reconnect: {e}"),
            );
        }
        Ok(())
    })
    .await
}

pub fn announce_connect_outcome(outcome: crate::server_link::restart::RestartOutcome) {
    use crate::cool_popup::{show_popup_as, PopupKind};
    use crate::server_link::restart::RestartOutcome;
    match outcome {
        RestartOutcome::Synced => {}
        RestartOutcome::AlreadyStopped => {
            show_popup_as(PopupKind::Warning, "Connected. Sync is still stopped.");
        }
        RestartOutcome::JustChanged => {
            show_popup_as(
                PopupKind::Warning,
                "Server dataset changed. Sync is stopped until you Reconcile or Adopt.",
            );
        }
        RestartOutcome::Failed(e) => {
            show_popup_as(PopupKind::Error, format!("Couldn't connect: {e}"));
        }
    }
}

pub async fn wipe_server(coms: ComsHandleForClient) -> Result<(), VerbError> {
    with_busy("Wiping server", async move {
        if crate::server_link::conn_state::current_conn_state()
            != crate::server_link::conn_state::ConnState::Connected
        {
            return Err(VerbError::refused("Can't wipe: not connected to the server"));
        }
        let queued_before = crate::server_link::pending_queue::pending_count_signal().get_untracked();
        sync_mode::set_sync_mode(SyncMode::NeedsReconcile);
        sync_mode::set_mode_reason(sync_mode::ModeReason::YouWiped);
        let result = coms.wipe_dataset().await;
        match result {
            Ok(payload) => {
                crate::server_link::dataset_generation::record_new_generation(payload.dataset_generation);
                crate::server_link::pending_queue::clear_pending_letters();
                crate::server_link::receiving::clear_seen_everywhere();
                crate::cool_popup::show_popup_as(
                    crate::cool_popup::PopupKind::Warning,
                    format!(
                        "Server wiped (generation {}). {} queued edits discarded; local pages kept. Sync stays stopped until you Reconcile or Adopt.",
                        payload.dataset_generation, queued_before,
                    ),
                );
                Ok(())
            }
            Err(e) => {
                sync_mode::set_mode_reason(sync_mode::ModeReason::WipeIncomplete);
                Err(VerbError::failed(format!(
                    "Wipe failed: {e}. Sync is now stopped; rerun Wipe, or Reconcile or Adopt."
                )))
            }
        }
    })
    .await
}

pub async fn adopt_local(coms: ComsHandleForClient) -> Result<(), VerbError> {
    with_busy("Adopting local data", async move {
        if crate::server_link::conn_state::current_conn_state()
            != crate::server_link::conn_state::ConnState::Connected
        {
            return Err(VerbError::refused("Can't adopt: not connected to the server"));
        }
        let queued_before = crate::server_link::pending_queue::pending_count_signal().get_untracked();
        let pages = crate::local_pages::read_local_pages_placeholder();
        let page_count = pages.len();
        let was_stopped = sync_mode::current_mode() != SyncMode::Connected;
        sync_mode::set_sync_mode(SyncMode::NeedsReconcile);
        let result = coms.adopt_dataset(pages).await;
        match result {
            Ok(payload) => {
                crate::server_link::dataset_generation::record_new_generation(payload.dataset_generation);
                crate::server_link::pending_queue::clear_pending_letters();
                crate::server_link::receiving::clear_seen_everywhere();
                sync_mode::set_sync_mode(SyncMode::Connected);
                let restart = crate::server_link::restart::restart_sync_stack(coms).await;
                let mut msg = format!(
                    "Adopted: the server now holds this client's {} pages (generation {}). {} queued edits discarded.",
                    page_count, payload.dataset_generation, queued_before,
                );
                if was_stopped {
                    msg.push_str(" Sync resumed.");
                }
                crate::cool_popup::show_popup_as(crate::cool_popup::PopupKind::Success, msg);
                if let crate::server_link::restart::RestartOutcome::Failed(e) = restart {
                    crate::cool_popup::show_popup_as(
                        crate::cool_popup::PopupKind::Warning,
                        format!("Adopted, but couldn't reconnect: {e}"),
                    );
                }
                Ok(())
            }
            Err(e) => {
                sync_mode::set_mode_reason(sync_mode::ModeReason::AdoptIncomplete);
                Err(VerbError::failed(format!(
                    "Adopt failed; the server may be partly written and adopt is safe to rerun: {e}"
                )))
            }
        }
    })
    .await
}

pub fn connect_to_server_and_announce(coms: ComsHandleForClient) {
    leptos::task::spawn_local(async move {
        let outcome = crate::server_link::restart::restart_sync_stack(coms).await;
        announce_connect_outcome(outcome);
    });
}
