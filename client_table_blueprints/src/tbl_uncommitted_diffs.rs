use protocol::new_table::{ColumnDef, ColumnType, id_column, not_null_col};

#[uniffi::export]
pub fn uncommitted_diffs_columns() -> Vec<ColumnDef> {
    vec![
        id_column(),
        not_null_col(ColumnType::Blob, "snapshot_of_edit"),
        not_null_col(ColumnType::Blob, "love_letter_sketch"),
        not_null_col(ColumnType::Text, "session_id"),
        not_null_col(ColumnType::Text, "target_id"),
    ]
}
