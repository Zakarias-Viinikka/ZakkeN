use crate::export_column_names_for_kotlin;
use crate::export_table_names_for_kotlin;
use protocol::new_table::{ColumnDef, ColumnType, default_col, id_column, not_null_unique_col};

export_table_names_for_kotlin!(
    pub const KEY_VALUE_STORAGE: &str = "key_value_storage",
);

export_column_names_for_kotlin!(
    key_value_storage,
    pub const KEY: &str = "key",
    pub const VALUE: &str = "value",
);

#[uniffi::export]
pub fn key_value_storage_columns() -> Vec<ColumnDef> {
    vec![
        id_column(),
        not_null_unique_col(ColumnType::Text, KEY),
        default_col(ColumnType::Text, VALUE),
    ]
}
