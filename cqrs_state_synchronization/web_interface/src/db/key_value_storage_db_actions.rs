//! Db reads and writes against the key_value_storage table.
//!
//! Two operations: read a value by key (None if absent), and write a
//! value by key (insert if the key isn't there, update if it is).
//! Nothing here caches — the cache lives in `cached_ids`, and the
//! callers in `ui_actions` and `init` set it after the write succeeds.

use client_table_blueprints::tbl_key_value_storage;
use error_stuff::CqrsErr;
use protocol::error::DbError;
use protocol::payload::*;
use protocol::row_col::Col;
use web_internal_db::db_helper;

pub async fn read_key_value_from_db(key: &str) -> Result<Option<String>, CqrsErr> {
    let out = db_helper::get_data(GetDataIn {
        table_name: tbl_key_value_storage::TABLE_NAME.into(),
        arguments: SelectArguments::Single(SelectArgument::XEqualY {
            x: tbl_key_value_storage::KEY.name.to_string(),
            y: key.to_string(),
        }),
        columns_to_read: vec![tbl_key_value_storage::VALUE.name.to_string()],
    })
    .await?;

    if out.rows.is_empty() {
        return Ok(None);
    }

    let mut cols = out.rows.into_iter().next().unwrap().cols.into_iter();
    let col = cols.next().ok_or_else(|| {
        CqrsErr::DbErrorContainer(DbError::IllegalInput(
            "read_key_value_from_db: row had no columns".to_string(),
        ))
    })?;

    match col {
        Col::Text(s) => Ok(Some(s)),
        Col::Null(_) => Ok(None),
        other => Err(CqrsErr::DbErrorContainer(DbError::IllegalInput(format!(
            "read_key_value_from_db: expected Text for key {key:?}, got {other:?}"
        )))),
    }
}

pub async fn write_key_value_to_db(key: &str, value: &str) -> Result<(), CqrsErr> {
    let existing = read_key_value_from_db(key).await?;

    if existing.is_some() {
        db_helper::edit_col_in_row_where(EditColInRowWhereIn {
            table_name: tbl_key_value_storage::TABLE_NAME.to_string(),
            where_clause: SelectArguments::Single(SelectArgument::XEqualY {
                x: tbl_key_value_storage::KEY.name.to_string(),
                y: key.to_string(),
            }),
            column: tbl_key_value_storage::VALUE.name.to_string(),
            new_value: Col::Text(value.to_string()),
        })
        .await?;
    } else {
        let values =
            tbl_key_value_storage::new_key_value_item(key.to_string(), value.to_string())?;
        db_helper::insert_data(InsertDataIn {
            table_name: tbl_key_value_storage::TABLE_NAME.into(),
            values,
        })
        .await?;
    }

    Ok(())
}
