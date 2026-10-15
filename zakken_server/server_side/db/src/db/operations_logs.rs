use client_table_blueprints::tbl_logs;
use db_wrapper::android_mascot::LiveForever;
use protocol::error::DbError;
use protocol::payload::{CreateTableIn, InsertDataIn};

pub fn create_table_if_not_exist(liver: &LiveForever) -> Result<(), DbError> {
    let list = liver.list_tables()?;
    if !list.table_names.iter().any(|t| t == tbl_logs::TABLE_NAME) {
        liver.create_table(CreateTableIn {
            table_name: tbl_logs::TABLE_NAME.to_string(),
            columns: tbl_logs::new_table_logs(),
        })?;
    }
    Ok(())
}

pub fn insert_log(
    liver: &LiveForever,
    timestamp: i64,
    level: &str,
    category: &str,
    source: &str,
    session_id: &str,
    message: &str,
) -> Result<(), DbError> {
    let row = tbl_logs::new_log_row(
        timestamp,
        level.to_string(),
        category.to_string(),
        source.to_string(),
        session_id.to_string(),
        message.to_string(),
    )
    .map_err(|e| DbError::BadCode(format!("couldn't build log row: {e:?}")))?;

    liver.insert_data(InsertDataIn {
        table_name: tbl_logs::TABLE_NAME.to_string(),
        values: row,
    })
}
