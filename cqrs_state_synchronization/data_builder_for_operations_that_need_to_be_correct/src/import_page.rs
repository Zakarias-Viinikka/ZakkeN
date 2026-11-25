//! Build a plan to create a page from blocks that already have ids.
//!
//! Pure. Makes a fresh yrs doc under the given page_id, inserts the
//! blocks in the order given, and hands the snapshot to
//! `build_merge_new_page`. Like a merge, it doesn't touch
//! client_love_letter. The executor runs the plan with `apply_merge_plan`.

use std::sync::Arc;

use error_stuff::CqrsErr;
use my_yrs_lib::yrs_wrapper::PositionToInsert;
use my_yrs_lib::{Block, BossOfYrs};

use crate::merge_page::{MergePlan, build_merge_new_page};

/// `blocks_in_order` must already be in page order, title first.
pub fn build_import_page(
    page_id: String,
    blocks_in_order: Vec<Block>,
    is_main_menu_page: bool,
    user_id: String,
    time: String,
) -> Result<MergePlan, CqrsErr> {
    let boss = Arc::new(BossOfYrs::new_with_preexisting_page_id(
        page_id.clone(),
        user_id.clone(),
        time.clone(),
    ));
    for block in blocks_in_order {
        boss.clone()
            .insert_new_block_with_preexisting_id(
                block.id_in_yrs,
                block.text,
                block.metadata,
                PositionToInsert::AtEnd,
            )
            .map_err(CqrsErr::YrsErrorContainer)?;
    }
    let snapshot = boss.snapshot().map_err(CqrsErr::YrsErrorContainer)?;
    build_merge_new_page(page_id, is_main_menu_page, snapshot, user_id, time)
}
