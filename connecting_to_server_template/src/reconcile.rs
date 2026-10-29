//! Reconcile verbs and state. Two-phase: plan (fetch server, classify, show
//! user) then apply (send decisions, replace local pages). Mirrors the real
//! reconcile flow in `plan_for_general_data_management_ops.md`.

use crate::client_server_coms::ComsHandleForClient;
use crate::protocol::{
    AdoptPage, LoveLetter, PageRow, CANNED_PAGE_ID_A, CANNED_PAGE_ID_B, CANNED_PAGE_ID_C,
};
use crate::server_link;
use crate::sync_mode::{self, SyncMode};
use leptos::prelude::*;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ConflictChoice {
    Server,
    Client,
}

pub type ReconcileDecisions = HashMap<String, ConflictChoice>;

#[derive(Clone, PartialEq, Debug)]
pub struct PageDiffSummary {
    pub page_id: String,
    pub client_title: String,
    pub server_title: String,
    pub only_client: usize,
    pub only_server: usize,
    pub changed: usize,
    pub moved: usize,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ReconcilePlan {
    pub take_from_server: Vec<String>,
    pub push_to_server: Vec<String>,
    pub identical: Vec<String>,
    pub disable_locally: Vec<String>,
    pub disable_on_server: Vec<String>,
    pub conflicts: Vec<PageDiffSummary>,
}

static RECONCILE_PLAN: LazyLock<ArcRwSignal<Option<ReconcilePlan>>> =
    LazyLock::new(|| ArcRwSignal::new(None));

static RECONCILE_ERROR: LazyLock<ArcRwSignal<Option<String>>> =
    LazyLock::new(|| ArcRwSignal::new(None));

struct ReconcileSession {
    previous_mode: SyncMode,
    previous_reason: Option<crate::sync_mode::ModeReason>,
    apply_attempted: bool,
    plan: ReconcilePlan,
}

static SESSION: LazyLock<Mutex<Option<ReconcileSession>>> =
    LazyLock::new(|| Mutex::new(None));

pub fn plan_signal() -> ArcRwSignal<Option<ReconcilePlan>> {
    RECONCILE_PLAN.clone()
}

pub fn error_signal() -> ArcRwSignal<Option<String>> {
    RECONCILE_ERROR.clone()
}

pub fn clear_modal() {
    RECONCILE_PLAN.set(None);
    RECONCILE_ERROR.set(None);
}

// PLACEHOLDER: the real `classify_pages` diffes each page's content and
// orders rows by (frac, id) on both sides. Here it branches on whether the
// server returned anything, so both modal shapes can be exercised.
fn classify_pages_placeholder(_local: &[AdoptPage], server: &[PageRow]) -> ReconcilePlan {
    if server.is_empty() {
        return ReconcilePlan {
            take_from_server: Vec::new(),
            push_to_server: vec![CANNED_PAGE_ID_A.to_string(), CANNED_PAGE_ID_B.to_string()],
            identical: Vec::new(),
            disable_locally: Vec::new(),
            disable_on_server: Vec::new(),
            conflicts: Vec::new(),
        };
    }
    ReconcilePlan {
        take_from_server: Vec::new(),
        push_to_server: Vec::new(),
        identical: vec![CANNED_PAGE_ID_C.to_string()],
        disable_locally: Vec::new(),
        disable_on_server: Vec::new(),
        conflicts: vec![
            PageDiffSummary {
                page_id: CANNED_PAGE_ID_A.to_string(),
                client_title: "Mine A".into(),
                server_title: "Server A".into(),
                only_client: 1,
                only_server: 0,
                changed: 2,
                moved: 1,
            },
            PageDiffSummary {
                page_id: CANNED_PAGE_ID_B.to_string(),
                client_title: "Mine B".into(),
                server_title: "Server B".into(),
                only_client: 0,
                only_server: 2,
                changed: 1,
                moved: 0,
            },
        ],
    }
}

pub async fn reconcile_plan(
    coms: ComsHandleForClient,
) -> Result<Option<ReconcilePlan>, crate::data_ops::VerbError> {
    use crate::data_ops::VerbError;
    if SESSION.lock().expect("session poisoned").is_some() {
        crate::cool_popup::show_popup("Reconcile already in progress.");
        return Ok(None);
    }
    crate::data_ops::with_busy("Fetching server pages…", async move {
        if server_link::current_conn_state() != server_link::ConnState::Connected {
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
            let local = crate::data_ops::read_local_pages_placeholder();
            Ok::<ReconcilePlan, VerbError>(classify_pages_placeholder(&local, &payload.pages))
        }
        .await;

        match result {
            Ok(plan) => {
                *SESSION.lock().expect("session poisoned") = Some(ReconcileSession {
                    previous_mode,
                    previous_reason,
                    apply_attempted: false,
                    plan: plan.clone(),
                });
                RECONCILE_ERROR.set(None);
                Ok(Some(plan))
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
) -> Result<(), crate::data_ops::VerbError> {
    use crate::data_ops::VerbError;
    crate::data_ops::with_busy("Applying reconcile", async move {
        if crate::server_link::current_restart_in_flight() {
            return Err(VerbError::refused("Still connecting."));
        }
        let plan = {
            let guard = SESSION.lock().expect("session poisoned");
            let session = guard
                .as_ref()
                .ok_or_else(|| VerbError::refused("no reconcile session"))?;
            session.plan.clone()
        };
        for conflict in &plan.conflicts {
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

        let discarded = server_link::pending_count_signal().get_untracked();
        let pushed = plan.push_to_server.len();
        let took = plan.take_from_server.len();
        let total_conflicts = plan.conflicts.len();
        let (a_client, b_server) = decisions.values().fold((0usize, 0usize), |(a, b), d| {
            match d {
                ConflictChoice::Client => (a + 1, b),
                ConflictChoice::Server => (a, b + 1),
            }
        });

        for page_id in &plan.push_to_server {
            coms.send_letter(LoveLetter::CreateNewPage {
                page_id: page_id.clone(),
                is_main_menu_page: false,
                snapshot_of_edit: b"pushed".to_vec(),
            })
            .await
            .map_err(|e| VerbError::failed(e.to_string()))?;
        }
        for conflict in &plan.conflicts {
            if decisions.get(&conflict.page_id) == Some(&ConflictChoice::Client) {
                coms.send_letter(LoveLetter::generic_merge(
                    conflict.page_id.clone(),
                    b"merged".to_vec(),
                ))
                .await
                .map_err(|e| VerbError::failed(e.to_string()))?;
            }
        }
        for page_id in &plan.disable_on_server {
            coms.send_letter(LoveLetter::DisablePage {
                page_id: page_id.clone(),
                snapshot_of_edit: b"disabled".to_vec(),
            })
            .await
            .map_err(|e| VerbError::failed(e.to_string()))?;
        }

        // PLACEHOLDER: the real local phase rebuilds the local mirror from
        // the plan and the captured blobs, inside one transaction.
        server_link::clear_pending_letters();

        if let Some(curr) = server_link::current_generation() {
            server_link::record_new_generation(curr);
        }
        sync_mode::set_sync_mode(SyncMode::Connected);
        *SESSION.lock().expect("session poisoned") = None;
        let _ = server_link::restart_sync_stack(coms).await;
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
) -> Result<(), crate::data_ops::VerbError> {
    crate::data_ops::with_busy("Cancelling reconcile…", async move {
        let restore = {
            let mut guard = SESSION.lock().expect("session poisoned");
            guard.take().map(|session| {
                if session.apply_attempted {
                    None
                } else {
                    Some((session.previous_mode, session.previous_reason))
                }
            })
        };
        match restore {
            Some(Some((prev_mode, prev_reason))) => {
                sync_mode::set_sync_mode(prev_mode);
                if let Some(reason) = prev_reason {
                    sync_mode::set_mode_reason(reason);
                }
            }
            Some(None) => {
                sync_mode::set_mode_reason(sync_mode::ModeReason::ReconcileIncomplete);
                crate::cool_popup::show_popup_as(
                    crate::cool_popup::PopupKind::Warning,
                    "Reconcile cancelled. Sync stays stopped (apply had started).",
                );
            }
            None => {}
        }
        let _ = server_link::restart_sync_stack(coms).await;
        Ok(())
    })
    .await
}
