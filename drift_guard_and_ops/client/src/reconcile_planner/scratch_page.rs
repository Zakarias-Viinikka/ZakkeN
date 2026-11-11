//! Caller pov: helpers around the scratch copy of the server's page.
//! The scratch copy is a writable `BossOfYrs` built from the server's
//! snapshot with a unique user id, so yrs never confuses its clock with
//! anything else's. All operations on it are local; nothing here talks
//! to a real server or the local DB.

use std::sync::Arc;

use my_yrs_lib::{
    doc_from_snapshot,
    yrs_wrapper::PositionToInsert,
    BossOfYrs,
};

pub fn open_a_scratch_copy_of_the_server_page(
    server_page: Arc<BossOfYrs>,
) -> Result<Arc<BossOfYrs>, String> {
    let page_id = server_page.clone().page_id();
    let blob = server_page
        .clone()
        .snapshot()
        .map_err(|e| format!("snapshot: {e:?}"))?;
    Ok(Arc::new(
        doc_from_snapshot(blob, scratch_user_id(), page_id, scratch_time())
            .map_err(|e| format!("doc_from_snapshot: {e:?}"))?,
    ))
}

/// `(block_id, text)` pairs, in visual order.
pub fn read_blocks_with_ids(scratch: &Arc<BossOfYrs>) -> Result<Vec<(String, String)>, String> {
    scratch
        .clone()
        .get_entire_page()
        .map_err(|e| format!("get_entire_page: {e:?}"))
        .map(|blocks| blocks.into_iter().map(|b| (b.id_in_yrs, b.text)).collect())
}

/// Where to insert a new block, given the id of the last block that was
/// just placed. `None` means "before the very first block". If the
/// scratch doc is empty or the last block is at the end, `AtEnd`.
pub fn position_for_insert(
    scratch: &Arc<BossOfYrs>,
    last_placed_id: Option<&str>,
) -> Result<PositionToInsert, String> {
    let all = read_blocks_with_ids(scratch)?;
    if all.is_empty() {
        return Ok(PositionToInsert::AtEnd);
    }
    match last_placed_id {
        None => Ok(PositionToInsert::Between {
            block_id_of_above: None,
            block_id_of_below: Some(all[0].0.clone()),
        }),
        Some(id) => {
            let idx = all
                .iter()
                .position(|(bid, _)| bid == id)
                .ok_or_else(|| format!("last_placed_id {id} not found in scratch"))?;
            if idx + 1 < all.len() {
                Ok(PositionToInsert::Between {
                    block_id_of_above: Some(id.to_string()),
                    block_id_of_below: Some(all[idx + 1].0.clone()),
                })
            } else {
                Ok(PositionToInsert::AtEnd)
            }
        }
    }
}

fn scratch_user_id() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("reconcile-{nanos}")
}

fn scratch_time() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_micros().to_string())
        .unwrap_or_else(|_| "0".to_string())
}
