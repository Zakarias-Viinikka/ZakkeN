use client_table_blueprints::new_row_helper::*;
use data_builder_for_operations_that_need_to_be_correct::{
    EverythingToInsertForNewPage,
    insert_structs::{BlocksToInsertCtx, PagesInsertCtx, UncommitedDiffsInsertCtx},
};
use error_stuff::cqrs_err::CqrsErr;
use protocol::payload::*;
use web_internal_db::db_helper;

#[macro_export]
macro_rules! unwrap_or_bail {
    ($method:expr) => {
        if let Err(result) = $method {
            web_internal_db::db_helper::regret_everything().await?;
            return Err(result.into());
        }
    };
}

pub async fn insert_page_requires_three_db_inserts(
    everything_to_insert_ctx: EverythingToInsertForNewPage,
) -> Result<String, CqrsErr> {
    //method returns the yrs id of the page

    let page_id_cloned = everything_to_insert_ctx.page_to_insert.page_id.clone();

    db_helper::begin_all_or_nothing().await?;
    unwrap_or_bail!(insert_into_pages(everything_to_insert_ctx.page_to_insert).await);
    unwrap_or_bail!(
        insert_into_every_block_in_existence(everything_to_insert_ctx.blocks_to_insert).await
    );
    unwrap_or_bail!(
        insert_into_uncommitted_diffs(everything_to_insert_ctx.uncommitted_diffs).await
    );
    db_helper::everything_went_perfectly().await?;
    Ok(page_id_cloned)
}

async fn insert_into_pages(insert_ctx: PagesInsertCtx) -> Result<(), CqrsErr> {
    let values = new_page_row(
        insert_ctx.page_id,
        insert_ctx.is_main_menu_page,
        insert_ctx.blobbed_page,
        insert_ctx.version,
    )?;

    db_helper::insert_data(InsertDataIn {
        table_name: "pages".to_string(),
        values,
    })
    .await?;

    Ok(())
}

async fn insert_into_every_block_in_existence(
    insert_ctx: BlocksToInsertCtx,
) -> Result<(), CqrsErr> {
    let title_values = new_every_block_in_existence_row(
        true,
        insert_ctx.title_block.is_part_of_main_menu_page,
        insert_ctx.title_block.content,
        insert_ctx.title_block.my_id_as_given_by_yrs,
        insert_ctx.title_block.id_of_page_i_belong_to,
        insert_ctx.title_block.position,
    )?;

    db_helper::insert_data(InsertDataIn {
        table_name: "every_block_in_existence".to_string(),
        values: title_values,
    })
    .await?;

    let normal_values = new_every_block_in_existence_row(
        false,
        insert_ctx.normal_block.is_part_of_main_menu_page,
        insert_ctx.normal_block.content,
        insert_ctx.normal_block.my_id_as_given_by_yrs,
        insert_ctx.normal_block.id_of_page_i_belong_to,
        insert_ctx.normal_block.position,
    )?;

    db_helper::insert_data(InsertDataIn {
        table_name: "every_block_in_existence".to_string(),
        values: normal_values,
    })
    .await?;

    Ok(())
}

async fn insert_into_uncommitted_diffs(
    insert_ctx: UncommitedDiffsInsertCtx,
) -> Result<(), CqrsErr> {
    let values = new_uncommitted_diff_row(
        insert_ctx.snapshot_of_edit,
        insert_ctx.love_letter_sketch,
        insert_ctx.session_id,
        insert_ctx.target_id,
    )?;

    db_helper::insert_data(InsertDataIn {
        table_name: "uncommitted_diffs".to_string(),
        values,
    })
    .await?;

    Ok(())
}
