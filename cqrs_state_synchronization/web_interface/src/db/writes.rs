use std::sync::Arc;

use data_builder_for_operations_that_need_to_be_correct::{
    add_block as build_add_block, edit_block as build_edit_block,
};
use error_stuff::CqrsErr;
use executor_of_what_the_builder_built_because_the_builder_shouldnt_touch_the_db::page_executor::{
    add_block_requires_three_db_actions, edit_block_requires_three_db_inserts,
};
use my_yrs_lib::yrs_wrapper::PositionToInsert;
use my_yrs_lib::BossOfYrs;
use text_diff::diff_logic::{get_diff, DiffResult};

use crate::db::yrs_errors::yrs_error_at;

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

pub async fn edit_title_for_page(
    yrs: Arc<BossOfYrs>,
    block_id: String,
    new_title: String,
    session_id: String,
) -> Result<(), CqrsErr> {
    let old_text = yrs.clone().read_block(block_id.clone())?.ok_or_else(|| {
        yrs_error_at(
            format!("no block with id: {block_id}"),
            "edit_title_for_page",
            file!(),
        )
    })?;
    let diff = get_diff(&old_text, &new_title);
    let ctx = match build_edit_block(yrs, block_id, diff, session_id)? {
        Some(ctx) => ctx,
        None => return Ok(()),
    };
    edit_block_requires_three_db_inserts(ctx).await?;
    Ok(())
}
