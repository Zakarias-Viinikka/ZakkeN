//! Android-facing entry points.
//!
//! Thin wrappers around the real builder functions. The real functions
//! return `CqrsErr`, which holds `YrsError`, `DbError` or `BrokenHeart`
//! inside it. Exporting that through uniffi would mean exporting all
//! three inner types recursively. Instead these wrappers convert any
//! `CqrsErr` into `BuilderErr`, which carries a single string.
//!
//! Rust callers keep using the real functions in `create_page`,
//! `edit_block` and `disable_page`. Android calls the wrappers here.
//! Nothing about the real functions or `CqrsErr` changes.

use std::sync::Arc;

use my_yrs_lib::BossOfYrs;
use my_yrs_lib::yrs_wrapper::PositionToInsert;
use text_diff::diff_logic::DiffResult;

use crate::add_block::{
    EverythingForAddBlock, add_block as real_add_block,
};
use crate::delete_block::{
    EverythingForDeleteBlock, delete_block as real_delete_block,
};
use crate::create_page::{
    EverythingToInsertForNewPage, create_page as real_create_page,
};
use crate::disable_page::{
    EverythingForDisablePage, disable_page as real_disable_page,
};
use crate::edit_block::{
    EverythingForEditBlock, edit_block as real_edit_block,
};
use crate::move_block::{
    EverythingForMoveBlock, move_block as real_move_block,
};

/// Error type Android sees. One string, no inner types. Rust code never
/// matches on this — it's constructed at the uniffi boundary.
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum BuilderErr {
    #[error("{0}")]
    Failed(String),
}

impl From<error_stuff::CqrsErr> for BuilderErr {
    fn from(e: error_stuff::CqrsErr) -> Self {
        BuilderErr::Failed(format!("{e:?}"))
    }
}

#[uniffi::export]
pub fn create_page(
    is_main_menu_page: bool,
    session_id: String,
) -> Result<EverythingToInsertForNewPage, BuilderErr> {
    real_create_page(is_main_menu_page, session_id).map_err(Into::into)
}

#[uniffi::export]
pub fn edit_block(
    yrs: Arc<BossOfYrs>,
    block_id: String,
    diff: DiffResult,
    session_id: String,
) -> Result<Option<EverythingForEditBlock>, BuilderErr> {
    real_edit_block(yrs, block_id, diff, session_id).map_err(Into::into)
}

#[uniffi::export]
pub fn add_block(
    yrs: Arc<BossOfYrs>,
    block_content: String,
    block_meta_data: String,
    position: PositionToInsert,
    is_part_of_main_menu_page: bool,
    session_id: String,
    insert_with_existing_block_id: Option<String>,
) -> Result<EverythingForAddBlock, BuilderErr> {
    real_add_block(
        yrs,
        block_content,
        block_meta_data,
        position,
        is_part_of_main_menu_page,
        session_id,
        insert_with_existing_block_id,
    )
    .map_err(Into::into)
}

#[uniffi::export]
pub fn delete_block(
    yrs: Arc<BossOfYrs>,
    block_id: String,
    session_id: String,
) -> Result<EverythingForDeleteBlock, BuilderErr> {
    real_delete_block(yrs, block_id, session_id).map_err(Into::into)
}

#[uniffi::export]
pub fn move_block(
    yrs: Arc<BossOfYrs>,
    block_id: String,
    block_id_of_above: Option<String>,
    block_id_of_below: Option<String>,
    session_id: String,
) -> Result<EverythingForMoveBlock, BuilderErr> {
    real_move_block(
        yrs,
        block_id,
        block_id_of_above,
        block_id_of_below,
        session_id,
    )
    .map_err(Into::into)
}

#[uniffi::export]
pub fn disable_page(
    page_id: String,
    session_id: String,
    active_pages_serialized_form: Vec<u8>,
) -> Result<EverythingForDisablePage, BuilderErr> {
    real_disable_page(page_id, session_id, active_pages_serialized_form).map_err(Into::into)
}
