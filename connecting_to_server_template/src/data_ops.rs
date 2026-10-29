//! Client-side verbs for general data management operations. Each verb
//! reads and writes the sync-mode signal and, when it needs the server,
//! goes through the coms handle. Mirrors the real UI contract in
//! `plan_for_general_data_management_ops.md`.

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

pub async fn with_busy<F, T>(message: impl Into<String>, fut: F) -> Result<T, String>
where
    F: std::future::Future<Output = Result<T, String>>,
{
    if DATASET_OP_BUSY.get_untracked().is_some() {
        return Err("another operation is running".into());
    }
    DATASET_OP_BUSY.set(Some(message.into()));
    let result = fut.await;
    DATASET_OP_BUSY.set(None);
    result
}

pub fn pause() -> Result<(), String> {
    if sync_mode::current_mode() == SyncMode::NeedsReconcile {
        return Err("sync is already stopped; reconcile or adopt first".into());
    }
    sync_mode::set_sync_mode(SyncMode::Paused);
    Ok(())
}

pub async fn resume(coms: ComsHandleForClient) -> Result<(), String> {
    if sync_mode::current_mode() != SyncMode::Paused {
        return Err("not paused".into());
    }
    sync_mode::set_sync_mode(SyncMode::Connected);
    crate::server_link::restart_sync_stack(coms).await
}

pub async fn wipe_server(coms: ComsHandleForClient) -> Result<(), String> {
    with_busy("wiping server", async move {
        crate::server_link::require_server()?;
        sync_mode::set_sync_mode(SyncMode::NeedsReconcile);
        let payload = coms.wipe_dataset().await.map_err(|e| e.to_string())?;
        crate::server_link::record_new_generation(payload.dataset_generation);
        crate::server_link::clear_pending_letters();
        crate::server_link::clear_seen_everywhere();
        crate::cool_popup::show_popup(
            "sync stays stopped until Reconcile or Adopt",
        );
        Ok(())
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

pub async fn adopt_local(coms: ComsHandleForClient) -> Result<(), String> {
    with_busy("adopting local data", async move {
        crate::server_link::require_server()?;
        sync_mode::set_sync_mode(SyncMode::NeedsReconcile);
        let pages = read_local_pages_placeholder();
        let result = coms.adopt_dataset(pages).await;
        match result {
            Ok(payload) => {
                crate::server_link::record_new_generation(payload.dataset_generation);
                crate::server_link::clear_pending_letters();
                crate::server_link::clear_seen_everywhere();
                sync_mode::set_sync_mode(SyncMode::Connected);
                match crate::server_link::restart_sync_stack(coms).await {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        crate::cool_popup::show_popup(format!("restart failed: {e}"));
                        Ok(())
                    }
                }
            }
            Err(e) => Err(format!(
                "adopt failed; the server may be partly written and adopt is safe to rerun: {e}"
            )),
        }
    })
    .await
}
