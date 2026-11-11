//! Caller pov: `finish_reconcile` runs the whole end-of-flow sequence.
//! Guard against local changes since mount, build letters via the
//! planner, send them as one batch, wipe, reload. Returns Err if
//! anything refuses; the caller shows it.

use std::collections::HashMap;
use std::sync::Arc;

use drift_guard_client::reconcile_planner::create_love_letters_out_of_reconcile;
use drift_guard_shared::fingerprint;
use love_letter::{build_a_love_letter, love_letter_to_bytes, LoveLetterSketch};
use my_yrs_lib::doc_from_snapshot;
use server_protocol::ServErr;

use crate::db::dev_actions::wipe_local_and_pull_from_server;
use crate::db::reads_wrapper::pages::{get_all_page_ids, get_yrs_unblobbed};
use crate::owner_of_interacting_with_server::reconcile_requests::send_reconcile_letters;

use super::phase::{ClientOnlyPage, MismatchedPage};
use super::resolve_choice::ResolveChoice;

pub async fn finish_reconcile(
    target_drift_version: String,
    target_reconcile_version: String,
    mismatched: Vec<MismatchedPage>,
    client_only: Vec<ClientOnlyPage>,
    choices: HashMap<String, ResolveChoice>,
    mount_fingerprints: HashMap<String, String>,
    user_id: String,
    time: String,
) -> Result<(), String> {
    verify_nothing_changed_since_mount(&mount_fingerprints, &user_id, &time).await?;

    let mut letters: Vec<Vec<u8>> = Vec::new();

    for page in &mismatched {
        let Some(choice) = choices.get(&page.page_id) else {
            return Err(format!("page {} has no choice", page.page_id));
        };
        let target_blocks = match choice {
            ResolveChoice::Mine => page.client_blocks.clone(),
            ResolveChoice::Server => continue,
            ResolveChoice::Merged(v) => v.clone(),
        };
        let server_boss = doc_from_snapshot(
            page.server_blob.clone(),
            user_id.clone(),
            page.page_id.clone(),
            time.clone(),
        )
        .map_err(|e| format!("decode server page {}: {e:?}", page.page_id))?;
        let page_letters =
            create_love_letters_out_of_reconcile(Arc::new(server_boss), target_blocks)?;
        for l in page_letters {
            letters.push(
                love_letter_to_bytes(&l)
                    .map_err(|e| format!("encode letter {}: {e:?}", page.page_id))?,
            );
        }
    }

    for co in &client_only {
        let sketch = LoveLetterSketch::CreateNewPage {
            page_id: co.page_id.clone(),
            is_main_menu_page: co.is_main_menu_page,
        };
        let letter = build_a_love_letter(sketch, co.client_blob.clone());
        letters.push(
            love_letter_to_bytes(&letter)
                .map_err(|e| format!("encode new-page letter {}: {e:?}", co.page_id))?,
        );
    }

    if !letters.is_empty() {
        send_reconcile_letters(target_drift_version, target_reconcile_version, letters)
            .await
            .map_err(|e| match e {
                ServErr::DriftVersionMismatch => {
                    "server dataset changed during reconcile; reload and start over".to_string()
                }
                other => format!("reconcile batch failed: {other:?}"),
            })?;
    }

    wipe_local_and_pull_from_server()
        .await
        .map_err(|e| format!("wipe after reconcile: {e:?}"))?;
    Ok(())
}

async fn verify_nothing_changed_since_mount(
    mount_fingerprints: &HashMap<String, String>,
    user_id: &str,
    time: &str,
) -> Result<(), String> {
    let now_ids = get_all_page_ids()
        .await
        .map_err(|e| format!("list local pages: {e:?}"))?;
    if now_ids.len() != mount_fingerprints.len() {
        return Err(
            "local page set changed since this page loaded; reload and start over".to_string(),
        );
    }
    for page_id in &now_ids {
        let boss = get_yrs_unblobbed(page_id.clone(), user_id.to_string(), time.to_string())
            .await
            .map_err(|e| format!("open local page {page_id}: {e:?}"))?;
        let blob = boss
            .clone()
            .snapshot()
            .map_err(|e| format!("snapshot {page_id}: {e:?}"))?;
        let current = fingerprint(blob.as_slice());
        match mount_fingerprints.get(page_id) {
            Some(mount) if *mount == current => {}
            _ => {
                return Err(format!(
                    "local page {page_id} changed since this page loaded; reload and start over"
                ));
            }
        }
    }
    Ok(())
}
