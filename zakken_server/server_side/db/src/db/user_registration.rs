use crate::db::schema::tbl_registered_users;
use db_wrapper::android_mascot::LiveForever;
use protocol::error::DbError;
use protocol::payload::{
    ColumnValue, CountRowsIn, CreateTableIn, InsertDataIn, SelectArgument, SelectArguments,
};
use protocol::row_col::Col;

pub fn create_table_if_not_exist(liver: &LiveForever) -> Result<(), DbError> {
    let list = liver.list_tables()?;
    if list
        .table_names
        .iter()
        .any(|t| t == tbl_registered_users::TABLE_NAME)
    {
        return Ok(());
    }

    liver.create_table(CreateTableIn {
        table_name: tbl_registered_users::TABLE_NAME.to_string(),
        columns: tbl_registered_users::new_table_registered_users(),
    })
}

pub fn user_exists(liver: &LiveForever, user_name: &str) -> Result<bool, DbError> {
    let out = liver.count_rows(CountRowsIn {
        table_name: tbl_registered_users::TABLE_NAME.to_string(),
        arguments: SelectArguments::Single(SelectArgument::XEqualY {
            x: tbl_registered_users::USER_NAME.name.to_string(),
            y: user_name.to_string(),
        }),
    })?;
    Ok(out.count > 0)
}

pub fn create_user(liver: &LiveForever, user_name: &str) -> Result<(), DbError> {
    if user_exists(liver, user_name)? {
        return Err(DbError::IllegalInput(format!(
            "user '{user_name}' is already registered"
        )));
    }

    liver.insert_data(InsertDataIn {
        table_name: tbl_registered_users::TABLE_NAME.to_string(),
        values: vec![ColumnValue {
            column_name: tbl_registered_users::USER_NAME.name.to_string(),
            value: Col::Text(user_name.to_string()),
        }],
    })
}