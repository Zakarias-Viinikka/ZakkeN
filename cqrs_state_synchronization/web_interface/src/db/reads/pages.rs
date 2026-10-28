use std::sync::Arc;

use client_table_blueprints::tbl_every_block_in_existence;
use client_table_blueprints::tbl_pages::{self, BLOBBED_PAGE, PAGE_ID};
use error_stuff::CqrsErr;
use my_yrs_lib::{doc_from_snapshot, BossOfYrs};
use protocol::error::DbError;
use protocol::payload::*;
use protocol::schema_helper::DestructDbReturnCol;
use web_internal_db::db_helper;

pub async fn get_yrs_unblobbed(
    page_id: String,
    user_id: String,
    time: String,
) -> Result<Arc<BossOfYrs>, DbError> {
    let out = db_helper::get_single_col(GetSingleColIn {
        table_name: tbl_pages::TABLE_NAME.into(),
        arguments: SelectArguments::Single(SelectArgument::XEqualY {
            x: PAGE_ID.name.to_string(),
            y: page_id.clone(),
        }),
        column_to_read: BLOBBED_PAGE.name.to_string(),
    })
    .await?;
    let blob = BLOBBED_PAGE
        .destruct_db_col(out.value)
        .ok_or_else(|| DbError::ColDestructFail("blobbed_page was null".to_string()))?;
    let boss = doc_from_snapshot(blob, user_id, page_id, time)
        .map_err(|e| DbError::IllegalInput(format!("doc_from_snapshot failed: {e:?}")))?;
    Ok(Arc::new(boss))
}

pub async fn page_exists(page_id: String) -> Result<bool, DbError> {
    let out = db_helper::get_data(GetDataIn {
        table_name: tbl_pages::TABLE_NAME.into(),
        arguments: SelectArguments::Single(SelectArgument::XEqualY {
            x: PAGE_ID.name.to_string(),
            y: page_id,
        }),
        columns_to_read: vec![PAGE_ID.name.to_string()],
    })
    .await?;
    Ok(!out.rows.is_empty())
}

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

pub async fn is_main_menu_page(page_id: String) -> Result<bool, CqrsErr> {
    let out = db_helper::get_single_col(GetSingleColIn {
        table_name: tbl_pages::TABLE_NAME.into(),
        arguments: SelectArguments::Single(SelectArgument::XEqualY {
            x: PAGE_ID.name.to_string(),
            y: page_id,
        }),
        column_to_read: tbl_pages::IS_MAIN_MENU_PAGE.name.to_string(),
    })
    .await?;
    let s = tbl_pages::IS_MAIN_MENU_PAGE
        .try_destruct_db_col(out.value)
        .map_err(|e| CqrsErr::DbErrorContainer(DbError::IllegalInput(e)))?
        .ok_or_else(|| {
            CqrsErr::DbErrorContainer(DbError::IllegalInput(
                "is_main_menu_page was null".to_string(),
            ))
        })?;
    Ok(s == "true")
}

pub async fn get_active_pages_blob(page_id: String) -> Result<Vec<u8>, CqrsErr> {
    let out = db_helper::get_single_col(GetSingleColIn {
        table_name: tbl_pages::TABLE_NAME.into(),
        arguments: SelectArguments::Single(SelectArgument::XEqualY {
            x: PAGE_ID.name.to_string(),
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
