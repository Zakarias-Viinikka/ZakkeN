use client_table_blueprints::tbl_key_value_storage;
use db_wrapper::android_mascot::LiveForever;
use protocol::error::DbError;
use protocol::payload::{
    CreateTableIn, DeleteRowWhereIn, GetDataIn, InsertDataIn, SelectArgument, SelectArguments,
};
use protocol::schema_helper::DestructDbReturnCol;

const SESSION_KEY: &str = "session_id";

pub fn create_table_if_not_exist(liver: &LiveForever) -> Result<(), DbError> {
    let list = liver.list_tables()?;
    if !list
        .table_names
        .iter()
        .any(|t| t == tbl_key_value_storage::TABLE_NAME)
    {
        liver.create_table(CreateTableIn {
            table_name: tbl_key_value_storage::TABLE_NAME.to_string(),
            columns: tbl_key_value_storage::new_table_key_value_storage(),
        })?;
    }
    Ok(())
}

/// Read a value by key. None if the key has never been set.
pub fn get_value(liver: &LiveForever, key: &str) -> Result<Option<String>, DbError> {
    let out = liver.get_data(GetDataIn {
        table_name: tbl_key_value_storage::TABLE_NAME.to_string(),
        arguments: SelectArguments::Single(SelectArgument::XEqualY {
            x: tbl_key_value_storage::KEY.name.to_string(),
            y: key.to_string(),
        }),
        columns_to_read: vec![tbl_key_value_storage::VALUE.name.to_string()],
    })?;

    let Some(row) = out.rows.into_iter().next() else {
        return Ok(None);
    };
    let Some(col) = row.cols.into_iter().next() else {
        return Ok(None);
    };
    let value = tbl_key_value_storage::VALUE
        .try_destruct_db_col(col)
        .map_err(DbError::BadCode)?
        .ok_or_else(|| DbError::BadCode("value was null".to_string()))?;
    Ok(Some(value))
}

fn insert_value(liver: &LiveForever, key: &str, value: &str) -> Result<(), DbError> {
    let row = tbl_key_value_storage::new_key_value_item(key.to_string(), value.to_string())
        .map_err(|e| DbError::BadCode(format!("couldn't build key_value row: {e:?}")))?;
    liver.insert_data(InsertDataIn {
        table_name: tbl_key_value_storage::TABLE_NAME.to_string(),
        values: row,
    })
}

/// Read the current session id, return the next one, and write it back.
/// First call ever returns 1. Every server start gets a distinct id, so a
/// log row's session_id tells you which run it came from.
pub fn bump_session_id(liver: &LiveForever) -> Result<i64, DbError> {
    let current = get_value(liver, SESSION_KEY)?
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);
    let next = current + 1;

    liver.delete_row_where(DeleteRowWhereIn {
        table_name: tbl_key_value_storage::TABLE_NAME.to_string(),
        arguments: SelectArguments::Single(SelectArgument::XEqualY {
            x: tbl_key_value_storage::KEY.name.to_string(),
            y: SESSION_KEY.to_string(),
        }),
    })?;
    insert_value(liver, SESSION_KEY, &next.to_string())?;

    Ok(next)
}
