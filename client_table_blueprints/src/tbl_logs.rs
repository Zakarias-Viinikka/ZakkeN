use protocol::new_table::{ColumnDef, ColumnType, id_column, not_null_col};

#[uniffi::export]
pub fn logs_columns() -> Vec<ColumnDef> {
    vec![
        id_column(),
        not_null_col(ColumnType::Integer, "timestamp"),
        not_null_col(ColumnType::Text, "level"),
        not_null_col(ColumnType::Text, "category"),
        not_null_col(ColumnType::Text, "source"),
        not_null_col(ColumnType::Text, "session_id"),
        not_null_col(ColumnType::Text, "message"),
        ColumnDef {
            name: "details".to_string(),
            column_type: "BLOB".to_string(),
            primary_key: false,
            not_null: false,
            unique: false,
            default_value: "".to_string(),
            autoincrement: false,
        },
        ColumnDef {
            name: "details_type".to_string(),
            column_type: "TEXT".to_string(),
            primary_key: false,
            not_null: false,
            unique: false,
            default_value: "".to_string(),
            autoincrement: false,
        },
    ]
}
