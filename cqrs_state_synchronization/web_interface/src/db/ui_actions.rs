use client_table_blueprints::{tbl_client_love_letter, tbl_every_block_in_existence, tbl_pages};
use data_builder_for_operations_that_need_to_be_correct::{
    add_block as build_add_block, create_page, delete_block as build_delete_block,
    disable_page as build_disable_page, edit_block as build_edit_block,
};
use error_stuff::CqrsErr;
use leptos::prelude::*;
use leptos::prelude::{GetUntracked, Set};
use executor_of_what_the_builder_built_because_the_builder_shouldnt_touch_the_db::page_executor::{
    add_block_requires_three_db_actions, delete_block_requires_three_db_actions,
    disabled_page_requires_three_db_actions, edit_block_requires_three_db_inserts,
    insert_page_requires_three_db_inserts,
};
use my_yrs_lib::yrs_wrapper::PositionToInsert;
use my_yrs_lib::BossOfYrs;
use protocol::payload::*;
use std::sync::Arc;
use text_diff::diff_logic::DiffResult;
use web_internal_db::db_helper;

use crate::complicated_signal_flow::BatcherHandle;
use crate::db::db_helpers_for_web_client;
use crate::db::ui_actions_helper::{get_active_pages_blob, get_page_is_disabled};

/// Add one block to a page. The caller already has the boss; it does
/// not need to be fetched here.
pub async fn add_block(
    batcher: BatcherHandle,
    yrs: Arc<BossOfYrs>,
    block_content: String,
    block_meta_data: String,
    position: PositionToInsert,
    is_part_of_main_menu_page: bool,
    session_id: String,
) -> Result<String, CqrsErr> {
    batcher.flush_now().await?;
    let ctx = build_add_block(
        yrs,
        block_content,
        block_meta_data,
        position,
        is_part_of_main_menu_page,
        session_id,
        None,
    )?;
    let block_id = ctx.every_block_in_existence_insert.my_id_as_given_by_yrs.clone();
    add_block_requires_three_db_actions(ctx).await?;
    Ok(block_id)
}

/// Add a block with a caller-chosen id, without flushing the batcher.
/// This exists so the ui can make blocks that the user can write to
/// before the db has made the block.
/// Only the batcher's own flush may call this. It already owns the
/// queue and is inside its transaction, and `add_block` above would
/// wait on the batcher and deadlock.
pub async fn add_block_unflushed(
    yrs: Arc<BossOfYrs>,
    block_id: String,
    block_content: String,
    block_meta_data: String,
    position: PositionToInsert,
    is_part_of_main_menu_page: bool,
    session_id: String,
) -> Result<(), CqrsErr> {
    let ctx = build_add_block(
        yrs,
        block_content,
        block_meta_data,
        position,
        is_part_of_main_menu_page,
        session_id,
        Some(block_id),
    )?;
    add_block_requires_three_db_actions(ctx).await?;
    Ok(())
}

/// Write one block edit to db. Unlike `edit_title_for_page`, this takes
/// the diff directly - the caller has already computed and squashed it.
/// The batcher uses this so pending text buffers are flushed with one
/// builder+executor call per block.
pub async fn edit_block(
    yrs: Arc<BossOfYrs>,
    block_id: String,
    diff: DiffResult,
    session_id: String,
) -> Result<(), CqrsErr> {
    let ctx = match build_edit_block(yrs, block_id, diff, session_id)? {
        Some(ctx) => ctx,
        None => return Ok(()),
    };
    edit_block_requires_three_db_inserts(ctx).await?;
    Ok(())
}

pub async fn delete_block(
    batcher: BatcherHandle,
    yrs: Arc<BossOfYrs>,
    block_id: String,
    session_id: String,
) -> Result<(), CqrsErr> {
    batcher.flush_now().await?;
    let ctx = build_delete_block(yrs, block_id, session_id)?;
    delete_block_requires_three_db_actions(ctx).await?;
    Ok(())
}

pub async fn set_user_id(name: String) -> Result<(), CqrsErr> {
    crate::db::key_value_storage_db_actions::write_key_value_to_db("user_id", &name).await?;
    crate::db::cached_ids::USER_ID.set(Some(name));
    Ok(())
}

async fn write_title_to_block(
    page_id: String,
    block_id: String,
    title: String,
    session_id: String,
) -> Result<(), CqrsErr> {
    let yrs = db_helpers_for_web_client::get_yrs_unblobbed(
        page_id,
        crate::db::cached_ids::USER_ID.get_untracked().expect(crate::db::cached_ids::INIT_ERR),
        crate::get_time(),
    )
    .await?;

    db_helpers_for_web_client::edit_title_for_page(yrs, block_id, title, session_id).await?;

    Ok(())
}

pub async fn create_new_page(batcher: BatcherHandle) -> Result<(), CqrsErr> {
    batcher.flush_now().await?;
    let session_id = crate::db::cached_ids::SESSION_ID.get_untracked().expect(crate::db::cached_ids::INIT_ERR);
    let everything = create_page(true, session_id.clone())?;

    let title_block_id = everything
        .blocks_to_insert
        .title_block
        .my_id_as_given_by_yrs
        .clone();
    let page_id = everything.page_to_insert.page_id.clone();

    insert_page_requires_three_db_inserts(everything).await?;

    write_title_to_block(page_id, title_block_id, "title".to_string(), session_id).await?;

    Ok(())
}

pub use crate::db::ui_actions_move_block_operations::{move_block, move_block_to_another_page};

pub async fn disable_page(
    batcher: BatcherHandle,
    page_id: String,
    session_id: String,
) -> Result<(), CqrsErr> {
    batcher.flush_now().await?;
    if get_page_is_disabled(page_id.clone()).await? {
        return Ok(());
    }

    let active_pages_blob = get_active_pages_blob(page_id.clone()).await?;

    let everything = build_disable_page(page_id, session_id, active_pages_blob)?;

    disabled_page_requires_three_db_actions(everything).await?;

    Ok(())
}

pub async fn delete_everything() -> Result<(), CqrsErr> {
    let tables_to_delete = vec![
        tbl_pages::TABLE_NAME,
        tbl_every_block_in_existence::TABLE_NAME,
        tbl_client_love_letter::TABLE_NAME,
    ];

    for table_name in &tables_to_delete {
        db_helper::delete_all_rows(DeleteAllRowsIn {
            table_name: table_name.to_string(),
        })
        .await?;
    }

    Ok(())
}