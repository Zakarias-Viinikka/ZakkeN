use data_builder_for_operations_that_need_to_be_correct::{
    EverythingForAddBlock, EverythingForDeleteBlock, EverythingForDisablePage,
    EverythingForEditBlock, EverythingForMoveBlock, EverythingToInsertForNewPage,
};
use error_stuff::CqrsErr;

use super::table_writes_internals::every_block::{
    delete_every_block_row, insert_into_every_block_in_existence,
    insert_single_every_block_row, mark_all_blocks_for_page_disabled,
    update_block_frac_position, update_every_block_content,
};
use super::table_writes_internals::love_letter::insert_into_client_love_letter;
use super::table_writes_internals::pages::{insert_into_pages, update_pages_blob, update_pages_status};
use super::transaction::run_in_transaction;

pub async fn insert_page_requires_three_db_inserts(
    everything_to_insert_ctx: EverythingToInsertForNewPage,
) -> Result<i64, CqrsErr> {
    run_in_transaction(|| async move {
        insert_into_pages(everything_to_insert_ctx.page_to_insert).await?;
        let title_db_id =
            insert_into_every_block_in_existence(everything_to_insert_ctx.blocks_to_insert)
                .await?;
        insert_into_client_love_letter(everything_to_insert_ctx.client_love_letter).await?;
        Ok(title_db_id)
    })
    .await
}

pub async fn edit_block_requires_three_db_inserts(
    edit_block_ctx: EverythingForEditBlock,
) -> Result<(), CqrsErr> {
    run_in_transaction(|| async move {
        update_pages_blob(edit_block_ctx.pages_update).await?;
        update_every_block_content(edit_block_ctx.every_block_in_existence_update).await?;
        insert_into_client_love_letter(edit_block_ctx.client_love_letter).await?;
        Ok(())
    })
    .await
}

pub async fn disabled_page_requires_three_db_actions(
    everything_to_disable: EverythingForDisablePage,
) -> Result<(), CqrsErr> {
    let EverythingForDisablePage {
        pages_update,
        client_love_letter,
    } = everything_to_disable;
    let page_id = pages_update.page_id;
    let new_active_pages_blob = pages_update.yrs_blob;
    let page_id_for_mark = page_id.clone();
    run_in_transaction(|| async move {
        update_pages_status(page_id, new_active_pages_blob).await?;
        mark_all_blocks_for_page_disabled(page_id_for_mark).await?;
        insert_into_client_love_letter(client_love_letter).await?;
        Ok(())
    })
    .await
}

pub async fn add_block_requires_three_db_actions(
    add_block_ctx: EverythingForAddBlock,
) -> Result<(), CqrsErr> {
    let EverythingForAddBlock {
        pages_update,
        every_block_in_existence_insert,
        client_love_letter,
    } = add_block_ctx;
    run_in_transaction(|| async move {
        update_pages_blob(pages_update).await?;
        insert_single_every_block_row(every_block_in_existence_insert).await?;
        insert_into_client_love_letter(client_love_letter).await?;
        Ok(())
    })
    .await
}

pub async fn move_block_requires_three_db_actions(
    move_block_ctx: EverythingForMoveBlock,
) -> Result<(), CqrsErr> {
    let EverythingForMoveBlock {
        pages_update,
        block_id,
        new_frac_position,
        client_love_letter,
    } = move_block_ctx;
    run_in_transaction(|| async move {
        update_pages_blob(pages_update).await?;
        update_block_frac_position(block_id, new_frac_position).await?;
        insert_into_client_love_letter(client_love_letter).await?;
        Ok(())
    })
    .await
}

pub async fn delete_block_requires_three_db_actions(
    delete_block_ctx: EverythingForDeleteBlock,
) -> Result<(), CqrsErr> {
    let EverythingForDeleteBlock {
        pages_update,
        block_id,
        client_love_letter,
    } = delete_block_ctx;
    run_in_transaction(|| async move {
        update_pages_blob(pages_update).await?;
        delete_every_block_row(block_id).await?;
        insert_into_client_love_letter(client_love_letter).await?;
        Ok(())
    })
    .await
}
