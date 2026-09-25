use crate::export_column_names_for_kotlin;
use crate::export_table_names_for_kotlin;
use protocol::new_table::{ColumnDef, ColumnType, id_column, not_null_col};

export_table_names_for_kotlin!(
    pub const UNCOMMITTED_DIFFS: &str = "uncommitted_diffs",
);

export_column_names_for_kotlin!(
    uncommitted_diffs,
    pub const SNAPSHOT_OF_EDIT: &str = "snapshot_of_edit",
    pub const LOVE_LETTER_SKETCH: &str = "love_letter_sketch",
    pub const SESSION_ID: &str = "session_id",
    pub const TARGET_ID: &str = "target_id",
);

#[uniffi::export]
pub fn uncommitted_diffs_columns() -> Vec<ColumnDef> {
    vec![
        id_column(),
        not_null_col(ColumnType::Blob, SNAPSHOT_OF_EDIT),
        not_null_col(ColumnType::Blob, LOVE_LETTER_SKETCH),
        not_null_col(ColumnType::Text, SESSION_ID),
        not_null_col(ColumnType::Text, TARGET_ID),
    ]
}
