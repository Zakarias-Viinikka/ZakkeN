//! Caller pov: `finish_reconcile` runs the end-of-flow sequence. Guard
//! against local changes since mount, encode the pre-built love letters,
//! send them as one batch, wipe, reload. Returns Err if anything refuses;
//! the caller shows it.
//!
//! The viewer produces the love letters per page; the caller builds any
//! additional letters for client-only pages and concatenates. This file
//! never touches a scratch doc, a sketch, or a years snapshot.

use std::collections::HashMap;

use drift_guard_shared::fingerprint;
use love_letter::{love_letter_to_bytes, LoveLetter};
use server_protocol::ServErr;

use crate::db::dev_actions::wipe_local_and_pull_from_server;
use crate::db::reads_wrapper::pages::{get_all_page_ids, get_yrs_unblobbed};
use crate::owner_of_interacting_with_server::reconcile_requests::send_reconcile_letters;

pub async fn finish_reconcile(
    target_drift_version: String,
    target_reconcile_version: String,
    letters: Vec<LoveLetter>,
    mount_fingerprints: HashMap<String, String>,
    user_id: String,
    time: String,
) -> Result<(), String> {
    verify_nothing_changed_since_mount(&mount_fingerprints, &user_id, &time).await?;

    let letters_as_bytes: Vec<Vec<u8>> = letters
        .iter()
        .map(love_letter_to_bytes)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("encode letter: {e:?}"))?;

    if !letters_as_bytes.is_empty() {
        send_reconcile_letters(
            target_drift_version,
            target_reconcile_version,
            letters_as_bytes,
        )
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