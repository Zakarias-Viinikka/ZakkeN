use std::sync::Arc;

use data_builder_for_operations_that_need_to_be_correct::{
    add_block as build_add_block, delete_block as build_delete_block,
};
use error_stuff::CqrsErr;
use executor_of_what_the_builder_built_because_the_builder_shouldnt_touch_the_db::page_executor::{
    add_block_requires_three_db_actions, delete_block_requires_three_db_actions,
};
use my_yrs_lib::yrs_wrapper::PositionToInsert;
use my_yrs_lib::BossOfYrs;

use crate::db::batcher::{run_exclusive, BatcherHandle};
use crate::db::main_send_unsent_love_letters::main_send_unsent_love_letters;

pub async fn add_block(
    batcher: BatcherHandle,
    yrs: Arc<BossOfYrs>,
    block_content: String,
    block_meta_data: String,
    position: PositionToInsert,
    is_part_of_main_menu_page: bool,
    session_id: String,
) -> Result<String, CqrsErr> {
    let block_id = run_exclusive(&batcher, || async move {
        let ctx = build_add_block(
            yrs,
            block_content,
            block_meta_data,
            position,
            is_part_of_main_menu_page,
            session_id,
            None,
        )?;
        let block_id = ctx
            .every_block_in_existence_insert
            .my_id_as_given_by_yrs
            .clone();
        add_block_requires_three_db_actions(ctx).await?;
        Ok(block_id)
    })
    .await?;
    main_send_unsent_love_letters();
    Ok(block_id)
}

pub async fn delete_block(
    batcher: BatcherHandle,
    yrs: Arc<BossOfYrs>,
    block_id: String,
    session_id: String,
) -> Result<(), CqrsErr> {
    run_exclusive(&batcher, || async move {
        let ctx = build_delete_block(yrs, block_id, session_id)?;
        delete_block_requires_three_db_actions(ctx).await?;
        Ok(())
    })
    .await?;
    main_send_unsent_love_letters();
    Ok(())
}