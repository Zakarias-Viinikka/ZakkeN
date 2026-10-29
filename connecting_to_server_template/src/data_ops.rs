//! Client-side verbs for general data management operations. Each verb
//! reads and writes the sync-mode signal and, when it needs the server,
//! goes through the coms handle. Mirrors the real UI contract in
//! `plan_for_general_data_management_ops.md`.

#[derive(Debug)]
pub enum VerbError {
    Refused(String),
    Failed(String),
}

impl VerbError {
    pub fn refused(msg: impl Into<String>) -> Self {
        VerbError::Refused(msg.into())
    }
    pub fn failed(msg: impl Into<String>) -> Self {
        VerbError::Failed(msg.into())
    }
}

impl std::fmt::Display for VerbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerbError::Refused(m) | VerbError::Failed(m) => write!(f, "{m}"),
        }
    }
}

use crate::client_server_coms::ComsHandleForClient;
use crate::protocol::AdoptPage;
use crate::sync_mode::{self, SyncMode};
use leptos::prelude::*;
use std::sync::LazyLock;

static DATASET_OP_BUSY: LazyLock<ArcRwSignal<Option<String>>> =
    LazyLock::new(|| ArcRwSignal::new(None));

pub fn dataset_op_busy() -> ArcRwSignal<Option<String>> {
    DATASET_OP_BUSY.clone()
}

pub async fn with_busy<F, T>(message: impl Into<String>, fut: F) -> Result<T, VerbError>
where
    F: std::future::Future<Output = Result<T, VerbError>>,
{
    if DATASET_OP_BUSY.get_untracked().is_some() {
        return Err(VerbError::refused("another operation is running"));
    }
    DATASET_OP_BUSY.set(Some(message.into()));
    let result = fut.await;
    DATASET_OP_BUSY.set(None);
    result
}

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
        if crate::server_link::current_conn_state()
            != crate::server_link::ConnState::Connected
        {
            return Err(VerbError::refused(
                "Can't wipe local: not connected to the server",
            ));
        }
        let queued_before = crate::server_link::pending_count_signal().get_untracked();
        let payload = coms
            .get_all_pages()
            .await
            .map_err(|e| VerbError::failed(e.to_string()))?;
        let page_count = payload.pages.len();
        crate::server_link::clear_pending_letters();
        crate::server_link::clear_seen_everywhere();
        crate::server_link::apply_server_pages_locally(&payload.pages);
        if let Some(curr) = crate::server_link::current_generation() {
            crate::server_link::record_new_generation(curr);
        }
        crate::sync_mode::set_sync_mode(SyncMode::Connected);
        let restart = crate::server_link::restart_sync_stack(coms).await;
        crate::cool_popup::show_popup_as(
            crate::cool_popup::PopupKind::Success,
            format!(
                "Wiped local. Pulled {} pages from server. {} queued edits discarded. Sync resumed.",
                page_count, queued_before,
            ),
        );
        if let crate::server_link::RestartOutcome::Failed(e) = restart {
            crate::cool_popup::show_popup_as(
                crate::cool_popup::PopupKind::Warning,
                format!("Wiped local, but couldn't reconnect: {e}"),
            );
        }
        Ok(())
    })
    .await
}

pub fn announce_connect_outcome(outcome: crate::server_link::RestartOutcome) {
    use crate::cool_popup::{show_popup_as, PopupKind};
    use crate::server_link::RestartOutcome;
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
        if crate::server_link::current_conn_state()
            != crate::server_link::ConnState::Connected
        {
            return Err(VerbError::refused("Can't wipe: not connected to the server"));
        }
        let queued_before = crate::server_link::pending_count_signal().get_untracked();
        sync_mode::set_sync_mode(SyncMode::NeedsReconcile);
        sync_mode::set_mode_reason(sync_mode::ModeReason::YouWiped);
        let result = coms.wipe_dataset().await;
        match result {
            Ok(payload) => {
                crate::server_link::record_new_generation(payload.dataset_generation);
                crate::server_link::clear_pending_letters();
                crate::server_link::clear_seen_everywhere();
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

// PLACEHOLDER: the real `read_local_pages_placeholder` reads every local
// page row (blobbed_page) from the db. Here it returns canned pages.
pub fn read_local_pages_placeholder() -> Vec<AdoptPage> {
    use crate::protocol::{CANNED_PAGE_ID_A, CANNED_PAGE_ID_B};
    vec![
        AdoptPage {
            page_id: CANNED_PAGE_ID_A.to_string(),
            is_main_menu_page: true,
            snapshot: b"local_a".to_vec(),
        },
        AdoptPage {
            page_id: CANNED_PAGE_ID_B.to_string(),
            is_main_menu_page: false,
            snapshot: b"local_b".to_vec(),
        },
    ]
}

pub async fn adopt_local(coms: ComsHandleForClient) -> Result<(), VerbError> {
    with_busy("Adopting local data", async move {
        if crate::server_link::current_conn_state()
            != crate::server_link::ConnState::Connected
        {
            return Err(VerbError::refused("Can't adopt: not connected to the server"));
        }
        let queued_before = crate::server_link::pending_count_signal().get_untracked();
        let pages = read_local_pages_placeholder();
        let page_count = pages.len();
        let was_stopped = sync_mode::current_mode() != SyncMode::Connected;
        sync_mode::set_sync_mode(SyncMode::NeedsReconcile);
        let result = coms.adopt_dataset(pages).await;
        match result {
            Ok(payload) => {
                crate::server_link::record_new_generation(payload.dataset_generation);
                crate::server_link::clear_pending_letters();
                crate::server_link::clear_seen_everywhere();
                sync_mode::set_sync_mode(SyncMode::Connected);
                let restart = crate::server_link::restart_sync_stack(coms).await;
                let mut msg = format!(
                    "Adopted: the server now holds this client's {} pages (generation {}). {} queued edits discarded.",
                    page_count, payload.dataset_generation, queued_before,
                );
                if was_stopped {
                    msg.push_str(" Sync resumed.");
                }
                crate::cool_popup::show_popup_as(crate::cool_popup::PopupKind::Success, msg);
                if let crate::server_link::RestartOutcome::Failed(e) = restart {
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
