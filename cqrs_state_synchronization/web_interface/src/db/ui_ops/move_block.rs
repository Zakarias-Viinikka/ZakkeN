use std::sync::Arc;

use data_builder_for_operations_that_need_to_be_correct::{
    add_block as build_add_block, delete_block as build_delete_block,
    move_block as build_move_block,
};
use error_stuff::CqrsErr;
use executor_of_what_the_builder_built_because_the_builder_shouldnt_touch_the_db::page_executor::{
    add_block_requires_three_db_actions, delete_block_requires_three_db_actions,
    move_block_requires_three_db_actions, run_in_transaction,
};
use my_yrs_lib::{yrs_wrapper::PositionToInsert, BossOfYrs};

use crate::db::batcher::{run_exclusive, BatcherHandle};
use crate::db::cached_ids::current_user_id;
use crate::db::main_send_unsent_love_letters::main_send_unsent_love_letters;
use crate::db::reads_wrapper::pages::{get_yrs_unblobbed, is_main_menu_page};
use crate::db::yrs_errors::yrs_error_at;
use crate::get_time;

pub async fn move_block(
    batcher: BatcherHandle,
    page_id: String,
    block_id: String,
    block_id_of_above: Option<String>,
    block_id_of_below: Option<String>,
    session_id: String,
) -> Result<(), CqrsErr> {
    run_exclusive(&batcher, || async move {
        let yrs = get_yrs_unblobbed(page_id, current_user_id(), get_time()).await?;
        let ctx =
            build_move_block(yrs, block_id, block_id_of_above, block_id_of_below, session_id)?;
        move_block_requires_three_db_actions(ctx).await?;
        Ok(())
    })
    .await?;
    main_send_unsent_love_letters();
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
    run_exclusive(&batcher, || async move {
        let content = old_boss
            .clone()
            .read_block(block_id.clone())?
            .ok_or_else(|| {
                yrs_error_at(
                    format!("no block with id: {block_id}"),
                    "move_block_to_another_page",
                    file!(),
                )
            })?;
        let meta = old_boss
            .clone()
            .read_block_meta(block_id.clone())?
            .ok_or_else(|| {
                yrs_error_at(
                    format!("no meta for block id: {block_id}"),
                    "move_block_to_another_page",
                    file!(),
                )
            })?;
        let is_main_menu_page = is_main_menu_page(new_boss.clone().page_id()).await?;
        let delete_ctx = build_delete_block(old_boss, block_id.clone(), session_id.clone())?;
        let add_ctx = build_add_block(
            new_boss,
            content,
            meta,
            PositionToInsert::Between {
                block_id_of_above: new_block_above_id,
                block_id_of_below: new_block_below_id,
            },
            is_main_menu_page,
            session_id,
            Some(block_id),
        )?;
        run_in_transaction(|| async move {
            delete_block_requires_three_db_actions(delete_ctx).await?;
            add_block_requires_three_db_actions(add_ctx).await?;
            Ok(())
        })
        .await
    })
    .await?;
    main_send_unsent_love_letters();
    Ok(())
}