use client_table_blueprints::tbl_every_block_in_existence;
use client_table_blueprints::tbl_pages;
use error_stuff::CqrsErr;
use protocol::error::DbError;
use protocol::payload::*;
use protocol::schema_helper::DestructDbReturnCol;
use web_internal_db::db_helper;

pub async fn get_page_is_disabled(page_id: String) -> Result<bool, CqrsErr> {
    let existing = db_helper::get_single_col(GetSingleColIn {
        table_name: tbl_every_block_in_existence::TABLE_NAME.into(),
        arguments: SelectArguments::Single(SelectArgument::XEqualY {
            x: tbl_every_block_in_existence::ID_OF_PAGE_I_BELONG_TO
                .name
                .to_string(),
            y: page_id,
        }),
        column_to_read: tbl_every_block_in_existence::PAGE_IS_DISABLED
            .name
            .to_string(),
    })
    .await?;

    let is_disabled = tbl_every_block_in_existence::PAGE_IS_DISABLED
        .try_destruct_db_col(existing.value)
        .map_err(|e| CqrsErr::DbErrorContainer(DbError::IllegalInput(e)))?
        .ok_or_else(|| {
            CqrsErr::DbErrorContainer(DbError::IllegalInput(
                "page_is_disabled was null".to_string(),
            ))
        })?;

    Ok(is_disabled == "true")
}

pub async fn get_active_pages_blob(page_id: String) -> Result<Vec<u8>, CqrsErr> {
    let out = db_helper::get_single_col(GetSingleColIn {
        table_name: tbl_pages::TABLE_NAME.into(),
        arguments: SelectArguments::Single(SelectArgument::XEqualY {
            x: tbl_pages::PAGE_ID.name.to_string(),
            y: page_id,
        }),
        column_to_read: tbl_pages::PAGE_STATUS.name.to_string(),
    })
    .await?;

    let blob = tbl_pages::PAGE_STATUS
        .try_destruct_db_col(out.value)
        .map_err(|e| CqrsErr::DbErrorContainer(DbError::IllegalInput(e)))?
        .ok_or_else(|| {
            CqrsErr::DbErrorContainer(DbError::IllegalInput(
                "db doesn't stop us from asking for the col for a row that doesn't exist"
                    .to_string(),
            ))
        })?;

    Ok(blob)
}
