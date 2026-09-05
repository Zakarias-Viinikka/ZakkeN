use protocol::new_table::{ColumnDef, ColumnType, id_column, not_null_col, not_null_unique_col};

#[uniffi::export]
pub fn pages_columns() -> Vec<ColumnDef> {
    vec![
        id_column(),
        not_null_unique_col(ColumnType::Text, "page_id"),
        not_null_col(ColumnType::Blob, "blobbed_page"),
        not_null_col(ColumnType::Blob, "page_status"),
        not_null_col(ColumnType::Blob, "version"),
        not_null_col(ColumnType::Text, "is_main_menu_page"),
    ]
}
