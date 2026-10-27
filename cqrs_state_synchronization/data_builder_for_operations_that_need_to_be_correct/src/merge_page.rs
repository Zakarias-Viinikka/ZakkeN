//! Build a plan to merge a remote snapshot into a page.
//!
//! Pure. Takes a boss, merges the snapshot, and returns a plan
//! describing what the mirror should look like after the merge. The
//! executor runs the plan.
//!
//! Doesn't touch client_love_letter: received letters must not echo.

use std::sync::Arc;

use error_stuff::CqrsErr;
use my_yrs_lib::yrs_active_pages::YrsActivePages;
use my_yrs_lib::yrs_wrapper::create_bookmark;
use my_yrs_lib::{doc_from_snapshot, BossOfYrs};

pub struct MergePlan {
    pub page_id: String,
    pub new_blob: Option<Vec<u8>>,
    pub new_status: Option<Vec<u8>>,
    pub set_page_disabled: bool,
    pub is_new_page: Option<NewPageInfo>,
    pub reconcile: Vec<ReconcileOp>,
    pub final_blocks: Vec<FinalBlock>,
    pub changed: bool,
    pub is_main_menu_page: bool,
}

pub struct NewPageInfo {
    pub is_main_menu_page: bool,
    pub version: Vec<u8>,
}

pub enum ReconcileOp {
    Insert {
        block_id: String,
        content: String,
        frac_position: String,
        is_title: bool,
    },
    UpdateContent {
        block_id: String,
        new_content: String,
    },
    UpdateFrac {
        block_id: String,
        new_frac_position: String,
    },
    Delete {
        block_id: String,
    },
}

#[derive(Clone)]
pub struct FinalBlock {
    pub block_id: String,
    pub content: String,
    pub is_title: bool,
}

pub struct ExistingBlock {
    pub block_id: String,
    pub content: String,
    pub frac_position: String,
    pub is_title: bool,
}

pub fn build_merge_blob(
    page_id: String,
    snapshot_of_edit: Vec<u8>,
    existing_blocks: Vec<ExistingBlock>,
    is_main_menu_page: bool,
    boss: Arc<BossOfYrs>,
) -> Result<MergePlan, CqrsErr> {
    boss.clone()
        .merge_with_snapshot(snapshot_of_edit)
        .map_err(CqrsErr::YrsErrorContainer)?;
    let new_blob = boss.clone().snapshot().map_err(CqrsErr::YrsErrorContainer)?;
    let (final_blocks, reconcile) = reconcile(&boss, &existing_blocks)?;
    let changed = !reconcile.is_empty();
    Ok(MergePlan {
        page_id,
        new_blob: Some(new_blob),
        new_status: None,
        set_page_disabled: false,
        is_new_page: None,
        reconcile,
        final_blocks,
        changed,
        is_main_menu_page,
    })
}

pub fn build_merge_disable(
    page_id: String,
    snapshot_of_edit: Vec<u8>,
    existing_status: Vec<u8>,
) -> Result<MergePlan, CqrsErr> {
    let active =
        Arc::new(YrsActivePages::new(existing_status).map_err(CqrsErr::YrsErrorContainer)?);
    YrsActivePages::merge_with_snapshot(active.clone(), snapshot_of_edit)
        .map_err(CqrsErr::YrsErrorContainer)?;
    let new_status = active.snapshot().map_err(CqrsErr::YrsErrorContainer)?;
    Ok(MergePlan {
        page_id,
        new_blob: None,
        new_status: Some(new_status),
        set_page_disabled: true,
        is_new_page: None,
        reconcile: Vec::new(),
        final_blocks: Vec::new(),
        changed: true,
        is_main_menu_page: false,
    })
}

pub fn build_merge_new_page(
    page_id: String,
    is_main_menu_page: bool,
    snapshot_of_edit: Vec<u8>,
    user_id: String,
    time: String,
) -> Result<MergePlan, CqrsErr> {
    let boss = Arc::new(
        doc_from_snapshot(snapshot_of_edit.clone(), user_id, page_id.clone(), time)
            .map_err(CqrsErr::YrsErrorContainer)?,
    );
    let version = create_bookmark(boss.clone()).map_err(CqrsErr::YrsErrorContainer)?;
    let (final_blocks, reconcile) = reconcile(&boss, &[])?;
    Ok(MergePlan {
        page_id,
        new_blob: Some(snapshot_of_edit),
        new_status: None,
        set_page_disabled: false,
        is_new_page: Some(NewPageInfo {
            is_main_menu_page,
            version,
        }),
        reconcile,
        final_blocks,
        changed: true,
        is_main_menu_page,
    })
}

fn reconcile(
    boss: &Arc<BossOfYrs>,
    existing: &[ExistingBlock],
) -> Result<(Vec<FinalBlock>, Vec<ReconcileOp>), CqrsErr> {
    let yrs_blocks = boss
        .clone()
        .get_entire_page()
        .map_err(CqrsErr::YrsErrorContainer)?;

    let mut ops = Vec::new();
    let mut final_blocks = Vec::with_capacity(yrs_blocks.len());
    let mut seen_ids = std::collections::HashSet::new();

    for (idx, b) in yrs_blocks.into_iter().enumerate() {
        let block_id = b.id_in_yrs.clone();
        let frac = boss
            .clone()
            .get_frac_position_of_block(block_id.clone())
            .map_err(CqrsErr::YrsErrorContainer)?;
        let is_title = idx == 0;
        seen_ids.insert(block_id.clone());

        match existing.iter().find(|e| e.block_id == block_id) {
            None => {
                ops.push(ReconcileOp::Insert {
                    block_id: block_id.clone(),
                    content: b.text.clone(),
                    frac_position: frac,
                    is_title,
                });
            }
            Some(existing_block) => {
                if existing_block.content != b.text {
                    ops.push(ReconcileOp::UpdateContent {
                        block_id: block_id.clone(),
                        new_content: b.text.clone(),
                    });
                }
                if existing_block.frac_position != frac {
                    ops.push(ReconcileOp::UpdateFrac {
                        block_id: block_id.clone(),
                        new_frac_position: frac,
                    });
                }
            }
        }

        final_blocks.push(FinalBlock {
            block_id,
            content: b.text,
            is_title,
        });
    }

    for e in existing {
        if !seen_ids.contains(&e.block_id) {
            ops.push(ReconcileOp::Delete {
                block_id: e.block_id.clone(),
            });
        }
    }

    Ok((final_blocks, ops))
}