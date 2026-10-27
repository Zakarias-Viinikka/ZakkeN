//! Move-block operations: same-page (reorder) and cross-page (relocate).
//!
//! Same-page is a frac rewrite inside one yrs doc — see `move_block`.
//! Cross-page leaves one yrs doc and lands in another. Two builders
//! and two executors, but one transaction: if either fails, both
//! pages roll back together.

use data_builder_for_operations_that_need_to_be_correct::{
    add_block as build_add_block, delete_block as build_delete_block,
    move_block as build_move_block,
};
use error_stuff::CqrsErr;

use leptos::prelude::GetUntracked;
use executor_of_what_the_builder_built_because_the_builder_shouldnt_touch_the_db::page_executor::{
    add_block_requires_three_db_actions, delete_block_requires_three_db_actions,
    move_block_requires_three_db_actions,
};
use my_yrs_lib::{BossOfYrs, YrsError, yrs_error::ErrorInfo, yrs_wrapper::PositionToInsert};
use std::sync::Arc;
use web_internal_db::db_helper;

use crate::complicated_signal_flow::BatcherHandle;
use crate::db::db_helpers_for_web_client;
use crate::db::ui_actions_helper::is_main_menu_page;
use crate::db::write_lock::WRITE_LOCK;

pub async fn move_block(
    batcher: BatcherHandle,
    page_id: String,
    block_id: String,
    block_id_of_above: Option<String>,
    block_id_of_below: Option<String>,
    session_id: String,
) -> Result<(), CqrsErr> {
    batcher.flush_now().await?;
    let _guard = WRITE_LOCK.acquire().await;
    let yrs = db_helpers_for_web_client::get_yrs_unblobbed(
        page_id,
        crate::db::cached_ids::USER_ID.get_untracked().expect(crate::db::cached_ids::INIT_ERR),
        crate::get_time(),
    )
    .await?;

    let everything = build_move_block(
        yrs,
        block_id,
        block_id_of_above,
        block_id_of_below,
        session_id,
    )?;

    move_block_requires_three_db_actions(everything).await?;

    drop(_guard);
    crate::server_link::kick_send();
    Ok(())
}

pub async fn move_block_to_another_page(
    batcher: BatcherHandle,
    old_boss: Arc<BossOfYrs>,
    new_boss: Arc<BossOfYrs>,
    block_id: String,
    new_block_above_id: Option<String>,
    new_block_below_id: Option<String>,
    session_id: String,
) -> Result<(), CqrsErr> {
    batcher.flush_now().await?;
    let _guard = WRITE_LOCK.acquire().await;
    let content = old_boss
        .clone()
        .read_block(block_id.clone())?
        .ok_or_else(|| {
            CqrsErr::YrsErrorContainer(YrsError::GenericError {
                info: ErrorInfo {
                    error_msg: format!("no block with id: {block_id}"),
                    file: file!().to_string(),
                    method: "move_block_to_another_page".to_string(),
                },
            })
        })?;
    let meta = old_boss
        .clone()
        .read_block_meta(block_id.clone())?
        .ok_or_else(|| {
            CqrsErr::YrsErrorContainer(YrsError::GenericError {
                info: ErrorInfo {
                    error_msg: format!("no meta for block id: {block_id}"),
                    file: file!().to_string(),
                    method: "move_block_to_another_page".to_string(),
                },
            })
        })?;

    let is_part_of_main_menu_page = is_main_menu_page(new_boss.clone().page_id()).await?;

    let delete_ctx = build_delete_block(old_boss, block_id.clone(), session_id.clone())?;

    let add_ctx = build_add_block(
        new_boss,
        content,
        meta,
        PositionToInsert::Between {
            block_id_of_above: new_block_above_id,
            block_id_of_below: new_block_below_id,
        },
        is_part_of_main_menu_page,
        session_id,
        Some(block_id),
    )?;

    db_helper::begin_all_or_nothing().await?;

    if let Err(e) = delete_block_requires_three_db_actions(delete_ctx).await {
        db_helper::regret_everything().await?;
        return Err(e);
    }

    if let Err(e) = add_block_requires_three_db_actions(add_ctx).await {
        db_helper::regret_everything().await?;
        return Err(e);
    }

    db_helper::everything_went_perfectly().await?;

    drop(_guard);
    crate::server_link::kick_send();
    Ok(())
}
