use protocol::error::DbError;
use protocol::new_table::{ColumnType, id_column, not_null_col};
use protocol::payload::{
    ColumnValue, CreateTableIn, EditColInRowIn, GetDataIn, InsertDataIn, SelectArgument,
    SelectArguments,
};
use protocol::row_col::Col;
use web_internal_db::db_helper;

const SCHEMA_VERSION_TABLE_NAME: &str = "schema_version";
const VERSION_COLUMN_NAME: &str = "version";

pub async fn ensure_schema_version_table_exists() -> Result<(), DbError> {
    let table_names = db_helper::list_tables().await?.table_names;
    if table_names
        .iter()
        .any(|name| name.as_str() == SCHEMA_VERSION_TABLE_NAME)
    {
        return Ok(());
    }

    db_helper::create_table(CreateTableIn {
        table_name: SCHEMA_VERSION_TABLE_NAME.to_string(),
        columns: vec![
            id_column(),
            not_null_col(ColumnType::Integer, VERSION_COLUMN_NAME),
        ],
    })
    .await
}

pub async fn read_stored_schema_version() -> Result<Option<i64>, DbError> {
    let out = db_helper::get_data(GetDataIn {
        table_name: SCHEMA_VERSION_TABLE_NAME.to_string(),
        arguments: SelectArguments::Single(SelectArgument::All),
        columns_to_read: vec![VERSION_COLUMN_NAME.to_string()],
    })
    .await?;

    let Some(row) = out.rows.into_iter().next() else {
        return Ok(None);
    };
    let Some(col) = row.cols.into_iter().next() else {
        return Err(DbError::BadCode(
            "web_migrations/db_actions_for_schema_version_table.rs, read_stored_schema_version: the schema_version row has no columns".to_string(),
        ));
    };
    match col {
        Col::Integer(version) => Ok(Some(version)),
        _ => Err(DbError::BadCode(
            "web_migrations/db_actions_for_schema_version_table.rs, read_stored_schema_version: the version column is not an integer".to_string(),
        )),
    }
}

pub async fn write_schema_version(new_version: i64) -> Result<(), DbError> {
    if read_stored_schema_version().await?.is_some() {
        db_helper::edit_col_in_row(EditColInRowIn {
            table_name: SCHEMA_VERSION_TABLE_NAME.to_string(),
            row_id: "1".to_string(),
            column: VERSION_COLUMN_NAME.to_string(),
            new_value: Col::Integer(new_version),
        })
        .await
    } else {
        db_helper::insert_data(InsertDataIn {
            table_name: SCHEMA_VERSION_TABLE_NAME.to_string(),
            values: vec![ColumnValue {
                column_name: VERSION_COLUMN_NAME.to_string(),
                value: Col::Integer(new_version),
            }],
        })
        .await
    }
}
