use protocol::new_table::{ColumnDef, ColumnType};

#[uniffi::export]
pub fn key_value_storage_columns() -> Vec<ColumnDef> {
    vec![
        ColumnDef {
            name: "key".to_string(),
            column_type: "TEXT".to_string(),
            primary_key: true,
            not_null: true,
            unique: false,
            default_value: "".to_string(),
            autoincrement: false,
        },
        ColumnDef {
            name: "value".to_string(),
            column_type: "TEXT".to_string(),
            primary_key: false,
            not_null: true,
            unique: false,
            default_value: "".to_string(),
            autoincrement: false,
        },
    ]
}
