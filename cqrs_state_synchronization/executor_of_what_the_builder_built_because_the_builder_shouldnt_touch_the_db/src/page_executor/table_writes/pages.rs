use client_table_blueprints::tbl_pages::{self, new_page_row};
use data_builder_for_operations_that_need_to_be_correct::{PagesInsertCtx, PagesUpdateCtx};
use error_stuff::CqrsErr;
use protocol::payload::InsertDataIn;
use protocol::row_col::Col;
use web_internal_db::db_helper;

use super::set_column_where;

pub async fn insert_into_pages(insert_ctx: PagesInsertCtx) -> Result<(), CqrsErr> {
    let values = new_page_row(
        insert_ctx.page_id,
        insert_ctx.is_main_menu_page,
        insert_ctx.blobbed_page,
        insert_ctx.version,
    )?;
    db_helper::insert_data(InsertDataIn {
        table_name: tbl_pages::TABLE_NAME.to_string(),
        values,
    })
    .await?;
    Ok(())
}

pub async fn update_pages_blob(update_ctx: PagesUpdateCtx) -> Result<(), CqrsErr> {
    set_column_where(
        tbl_pages::TABLE_NAME,
        tbl_pages::PAGE_ID.name,
        update_ctx.page_id,
        tbl_pages::BLOBBED_PAGE.name,
        Col::Blob(update_ctx.yrs_blob),
    )
    .await
}

pub async fn update_pages_status(
    page_id: String,
    new_active_pages_blob: Vec<u8>,
) -> Result<(), CqrsErr> {
    set_column_where(
        tbl_pages::TABLE_NAME,
        tbl_pages::PAGE_ID.name,
        page_id,
        tbl_pages::PAGE_STATUS.name,
        Col::Blob(new_active_pages_blob),
    )
    .await
}
