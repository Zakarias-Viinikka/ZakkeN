use protocol::new_table::{ColumnDef, ColumnType, id_column, not_null_col};

#[uniffi::export]
pub fn uncommitted_diffs_columns() -> Vec<ColumnDef> {
    vec![
        id_column(),
        not_null_col(ColumnType::Blob, "snapshot_of_edit"),
        not_null_col(ColumnType::Blob, "edit_enum"),
        not_null_col(ColumnType::Integer, "session_id"),
        not_null_col(ColumnType::Text, "target_id"),
    ]
}
