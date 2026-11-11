//! Caller pov: read the client's pages, fetch the server's fingerprints,
//! fetch the two version numbers, and return the classification the
//! page renders from. All the network and DB work happens here.

use std::collections::HashMap;

use drift_guard_shared::fingerprint;

use crate::db::cached_ids::current_user_id;
use crate::db::reads_wrapper::pages::{get_all_page_ids, get_yrs_unblobbed, is_main_menu_page};
use crate::get_time;
use crate::owner_of_interacting_with_server::reconcile_requests::{
    fetch_page_fingerprints, fetch_pages_by_ids, fetch_reconcile_version,
};

use super::phase::{ClientOnlyPage, MismatchedPage, Phase};

pub async fn load_and_classify() -> Result<Phase, String> {
    let user_id = current_user_id();
    let time = get_time();

    let local = read_local_pages(&user_id, &time).await?;
    let server_fps = fetch_page_fingerprints()
        .await
        .map_err(|e| format!("fetch fingerprints: {e:?}"))?;
    let target_drift_version = server_fps.drift_version.clone();
    let server_fp_map: HashMap<String, String> = server_fps
        .pages
        .into_iter()
        .map(|p| (p.page_id, p.fingerprint))
        .collect();

    let reconcile_version = fetch_reconcile_version()
        .await
        .map_err(|e| format!("fetch reconcile version: {e:?}"))?;

    let mut mount_fingerprints: HashMap<String, String> = HashMap::new();
    for page_id in &local.ids {
        mount_fingerprints.insert(
            page_id.clone(),
            fingerprint(local.blobs.get(page_id).map(Vec::as_slice).unwrap_or(&[])),
        );
    }

    let mut mismatched_ids: Vec<String> = Vec::new();
    let mut client_only_ids: Vec<String> = Vec::new();
    for page_id in &local.ids {
        match server_fp_map.get(page_id) {
            Some(server_fp) => {
                if mount_fingerprints.get(page_id) != Some(server_fp) {
                    mismatched_ids.push(page_id.clone());
                }
            }
            None => client_only_ids.push(page_id.clone()),
        }
    }

    let mismatched =
        fetch_and_pair_mismatched(mismatched_ids, &local.blocks, &local.blobs, &user_id, &time)
            .await?;

    let mut client_only: Vec<ClientOnlyPage> = Vec::new();
    for page_id in client_only_ids {
        let blob = local.blobs.get(&page_id).cloned().unwrap_or_default();
        let is_menu = is_main_menu_page(page_id.clone())
            .await
            .map_err(|e| format!("is_main_menu_page {page_id}: {e:?}"))?;
        client_only.push(ClientOnlyPage {
            page_id,
            client_blob: blob,
            is_main_menu_page: is_menu,
        });
    }

    Ok(Phase::Ready {
        target_drift_version,
        target_reconcile_version: reconcile_version.reconcile_version,
        mismatched,
        client_only,
        mount_fingerprints,
        user_id,
        time,
    })
}

struct LocalPages {
    ids: Vec<String>,
    blobs: HashMap<String, Vec<u8>>,
    blocks: HashMap<String, Vec<String>>,
}

async fn read_local_pages(user_id: &str, time: &str) -> Result<LocalPages, String> {
    let ids = get_all_page_ids()
        .await
        .map_err(|e| format!("list local pages: {e:?}"))?;
    let mut blobs = HashMap::new();
    let mut blocks = HashMap::new();
    for page_id in &ids {
        let boss = get_yrs_unblobbed(page_id.clone(), user_id.to_string(), time.to_string())
            .await
            .map_err(|e| format!("open local page {page_id}: {e:?}"))?;
        let blob = boss
            .clone()
            .snapshot()
            .map_err(|e| format!("snapshot {page_id}: {e:?}"))?;
        let text_list: Vec<String> = boss
            .clone()
            .get_entire_page()
            .map_err(|e| format!("read blocks {page_id}: {e:?}"))?
            .into_iter()
            .map(|b| b.text)
            .collect();
        blobs.insert(page_id.clone(), blob);
        blocks.insert(page_id.clone(), text_list);
    }
    Ok(LocalPages { ids, blobs, blocks })
}

async fn fetch_and_pair_mismatched(
    mismatched_ids: Vec<String>,
    client_blocks: &HashMap<String, Vec<String>>,
    client_blobs: &HashMap<String, Vec<u8>>,
    user_id: &str,
    time: &str,
) -> Result<Vec<MismatchedPage>, String> {
    if mismatched_ids.is_empty() {
        return Ok(Vec::new());
    }
    let server_pages = fetch_pages_by_ids(mismatched_ids.clone())
        .await
        .map_err(|e| format!("fetch pages by ids: {e:?}"))?
        .pages;
    let server_blob_by_id: HashMap<String, Vec<u8>> = server_pages
        .into_iter()
        .map(|p| (p.page_id, p.blobbed_page))
        .collect();

    let mut result = Vec::new();
    for page_id in mismatched_ids {
        let Some(server_blob) = server_blob_by_id.get(&page_id) else {
            continue;
        };
        let server_blocks = block_texts_from_blob(server_blob, user_id, time, &page_id)?;
        let client = client_blocks.get(&page_id).cloned().unwrap_or_default();
        let client_blob = client_blobs.get(&page_id).cloned().unwrap_or_default();
        result.push(MismatchedPage {
            page_id,
            client_blob,
            client_blocks: client,
            server_blob: server_blob.clone(),
            server_blocks,
        });
    }
    Ok(result)
}

fn block_texts_from_blob(
    blob: &[u8],
    user_id: &str,
    time: &str,
    page_id: &str,
) -> Result<Vec<String>, String> {
    let boss = my_yrs_lib::doc_from_snapshot(
        blob.to_vec(),
        user_id.to_string(),
        page_id.to_string(),
        time.to_string(),
    )
    .map_err(|e| format!("decode server page {page_id}: {e:?}"))?;
    let blocks = std::sync::Arc::new(boss)
        .get_entire_page()
        .map_err(|e| format!("read server blocks {page_id}: {e:?}"))?;
    Ok(blocks.into_iter().map(|b| b.text).collect())
}
