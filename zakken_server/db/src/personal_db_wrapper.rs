use crate::db::schema::tbl_socket_testing_table;
use db_wrapper::android_mascot::LiveForever;
use protocol::error::DbError;
use protocol::payload::{
    ColumnValue, CreateTableIn, GetDataIn, InsertDataIn, SelectArgument, SelectArguments,
};
use protocol::row_col::Col;

const FILEPATH: &str = "test_db.sqlite";
const TABLE_NAME: &str = "socket_testing_table";

pub fn new_liver() -> Result<LiveForever, DbError> {
    LiveForever::new(FILEPATH)
}

pub fn create_table_if_not_exist(liver: &LiveForever) -> Result<(), DbError> {
    let list = liver.list_tables()?;
    if list.table_names.iter().any(|t| t == TABLE_NAME) {
        return Ok(());
    }

    let columns = tbl_socket_testing_table::column_definitions();
    liver.create_table(CreateTableIn {
        table_name: TABLE_NAME.to_string(),
        columns,
    })
}

pub fn create_data_if_not_exist(liver: &LiveForever) -> Result<(), DbError> {
    if !any_rows_exist(liver)? {
        let column_name = tbl_socket_testing_table::COLUMN_NAME;

        let mut stuff_to_insert = Vec::new();
        stuff_to_insert.push(ColumnValue {
            column_name: column_name.to_string(),
            value: Col::Text("text1".to_string()),
        });
        stuff_to_insert.push(ColumnValue {
            column_name: column_name.to_string(),
            value: Col::Text("text2".to_string()),
        });
        stuff_to_insert.push(ColumnValue {
            column_name: column_name.to_string(),
            value: Col::Text("text3".to_string()),
        });

        for inserty_werty in stuff_to_insert {
            liver.insert_data(InsertDataIn {
                table_name: TABLE_NAME.to_string(),
                values: vec![inserty_werty],
            })?;
        }
    }
    Ok(())
}

fn any_rows_exist(liver: &LiveForever) -> Result<bool, DbError> {
    let ctx = GetDataIn {
        table_name: TABLE_NAME.to_string(),
        arguments: SelectArguments::Single(SelectArgument::All),
        columns_to_read: Vec::new(),
    };
    let out = liver.get_data(ctx)?;
    Ok(!out.rows.is_empty())
}