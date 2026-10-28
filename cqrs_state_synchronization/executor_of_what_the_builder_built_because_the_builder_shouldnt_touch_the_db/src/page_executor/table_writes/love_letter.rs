use client_table_blueprints::tbl_client_love_letter::{self, new_client_love_letter_row};
use data_builder_for_operations_that_need_to_be_correct::ClientLoveLetterInsertCtx;
use error_stuff::CqrsErr;
use protocol::payload::InsertDataIn;
use web_internal_db::db_helper;

pub async fn insert_into_client_love_letter(
    insert_ctx: ClientLoveLetterInsertCtx,
) -> Result<(), CqrsErr> {
    let values = new_client_love_letter_row(
        insert_ctx.snapshot_of_edit,
        insert_ctx.love_letter_sketch,
        insert_ctx.session_id,
        insert_ctx.target_id,
    )?;
    db_helper::insert_data(InsertDataIn {
        table_name: tbl_client_love_letter::TABLE_NAME.into(),
        values,
    })
    .await?;
    Ok(())
}
