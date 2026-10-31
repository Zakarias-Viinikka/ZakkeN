//! Reconcile verbs and state. Two-phase: plan (fetch server, classify, show
//! user) then apply (send decisions, replace local pages). Mirrors the real
//! reconcile flow in `plan_for_general_data_management_ops.md`.

pub mod classify;
pub mod modal;
pub mod plan;
pub mod session;

use crate::client_server_coms::ComsHandleForClient;
use crate::protocol::LoveLetter;
use crate::owner_of_interacting_with_server;
use crate::sync_mode::{self, SyncMode};
use leptos::prelude::*;
use session::{RestoreAction, ReconcileSession, SESSION};

pub use modal::{clear_modal, error_signal, plan_signal};
pub use plan::{ConflictChoice, PageDiffSummary, ReconcileDecisions, ReconcilePlan};

pub async fn reconcile_plan(
    coms: ComsHandleForClient,
) -> Result<Option<plan::ReconcilePlan>, crate::data_ops::verb_error::VerbError> {
    use crate::data_ops::verb_error::VerbError;
    if SESSION.lock().expect("session poisoned").is_some() {
        crate::cool_popup::show_popup("Reconcile already in progress.");
        return Ok(None);
    }
    crate::data_ops::busy::with_busy("Fetching server pages…", async move {
        if owner_of_interacting_with_server::conn_state::current_conn_state()
            != owner_of_interacting_with_server::conn_state::ConnState::Connected
        {
            return Err(VerbError::refused("Can't reconcile: not connected to the server"));
        }
        let previous_mode = sync_mode::current_mode();
        let previous_reason = sync_mode::current_mode_reason();
        sync_mode::set_sync_mode(SyncMode::NeedsReconcile);
        sync_mode::set_mode_reason(sync_mode::ModeReason::Reconciling);

        let result = async {
            let payload = coms
                .get_all_pages()
                .await
                .map_err(|e| VerbError::failed(e.to_string()))?;
            let local = crate::local_pages::read_local_pages_placeholder();
            Ok::<plan::ReconcilePlan, VerbError>(
                classify::classify_pages_placeholder(&local, &payload.pages),
            )
        }
        .await;

        match result {
            Ok(p) => {
                *SESSION.lock().expect("session poisoned") = Some(ReconcileSession {
                    previous_mode,
                    previous_reason,
                    apply_attempted: false,
                    plan: p.clone(),
                });
                modal::error_signal().set(None);
                Ok(Some(p))
            }
            Err(e) => {
                sync_mode::set_sync_mode(previous_mode);
                if let Some(r) = previous_reason {
                    sync_mode::set_mode_reason(r);
                }
                Err(e)
            }
        }
    })
    .await
}

pub async fn reconcile_apply(
    coms: ComsHandleForClient,
    decisions: ReconcileDecisions,
) -> Result<(), crate::data_ops::verb_error::VerbError> {
    use crate::data_ops::verb_error::VerbError;
    use plan::ConflictChoice;
    crate::data_ops::busy::with_busy("Applying reconcile", async move {
        if crate::owner_of_interacting_with_server::restart::current_restart_in_flight() {
            return Err(VerbError::refused("Still connecting."));
        }
        let the_plan = {
            let guard = SESSION.lock().expect("session poisoned");
            let session = guard
                .as_ref()
                .ok_or_else(|| VerbError::refused("no reconcile session"))?;
            session.plan.clone()
        };
        for conflict in &the_plan.conflicts {
            if !decisions.contains_key(&conflict.page_id) {
                return Err(VerbError::refused(format!(
                    "missing decision for {}",
                    conflict.page_id
                )));
            }
        }
        {
            let mut guard = SESSION.lock().expect("session poisoned");
            if let Some(session) = guard.as_mut() {
                session.apply_attempted = true;
            }
        }

        let discarded = owner_of_interacting_with_server::pending_queue::pending_count_signal().get_untracked();
        let pushed = the_plan.push_to_server.len();
        let took = the_plan.take_from_server.len();
        let total_conflicts = the_plan.conflicts.len();
        let (a_client, b_server) = decisions.values().fold((0usize, 0usize), |(a, b), d| {
            match d {
                ConflictChoice::Client => (a + 1, b),
                ConflictChoice::Server => (a, b + 1),
            }
        });

        for page_id in &the_plan.push_to_server {
            coms.send_letter(LoveLetter::CreateNewPage {
                page_id: page_id.clone(),
                is_main_menu_page: false,
                snapshot_of_edit: b"pushed".to_vec(),
            })
            .await
            .map_err(|e| VerbError::failed(e.to_string()))?;
        }
        for conflict in &the_plan.conflicts {
            if decisions.get(&conflict.page_id) == Some(&ConflictChoice::Client) {
                coms.send_letter(LoveLetter::generic_merge(
                    conflict.page_id.clone(),
                    b"merged".to_vec(),
                ))
                .await
                .map_err(|e| VerbError::failed(e.to_string()))?;
            }
        }
        for page_id in &the_plan.disable_on_server {
            coms.send_letter(LoveLetter::DisablePage {
                page_id: page_id.clone(),
                snapshot_of_edit: b"disabled".to_vec(),
            })
            .await
            .map_err(|e| VerbError::failed(e.to_string()))?;
        }

        // PLACEHOLDER: the real local phase rebuilds the local mirror from
        // the plan and the captured blobs, inside one transaction.
        owner_of_interacting_with_server::pending_queue::clear_pending_letters();

        if let Some(curr) = owner_of_interacting_with_server::dataset_generation::current_generation() {
            owner_of_interacting_with_server::dataset_generation::record_new_generation(curr);
        }
        sync_mode::set_sync_mode(SyncMode::Connected);
        *SESSION.lock().expect("session poisoned") = None;
        let _ = owner_of_interacting_with_server::restart::restart_sync_stack(coms).await;
        crate::cool_popup::show_popup_as(
            crate::cool_popup::PopupKind::Success,
            format!(
                "Reconciled: pushed {}, took {} from server, resolved {} conflicts ({} yours, {} server's). {} queued edits discarded.",
                pushed, took, total_conflicts, a_client, b_server, discarded,
            ),
        );
        Ok(())
    })
    .await
}

pub async fn reconcile_cancel(
    coms: ComsHandleForClient,
) -> Result<(), crate::data_ops::verb_error::VerbError> {
    crate::data_ops::busy::with_busy("Cancelling reconcile…", async move {
        match session::take_session_for_cancel() {
            RestoreAction::Restore(prev_mode, prev_reason) => {
                sync_mode::set_sync_mode(prev_mode);
                if let Some(reason) = prev_reason {
                    sync_mode::set_mode_reason(reason);
                }
            }
            RestoreAction::ApplyHadStarted => {
                sync_mode::set_mode_reason(sync_mode::ModeReason::ReconcileIncomplete);
                crate::cool_popup::show_popup_as(
                    crate::cool_popup::PopupKind::Warning,
                    "Reconcile cancelled. Sync stays stopped (apply had started).",
                );
            }
            RestoreAction::NoSession => {}
        }
        let _ = owner_of_interacting_with_server::restart::restart_sync_stack(coms).await;
        Ok(())
    })
    .await
}