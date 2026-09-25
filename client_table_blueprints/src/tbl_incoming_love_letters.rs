use crate::export_column_names_for_kotlin;
use crate::export_table_names_for_kotlin;
use protocol::new_table::{ColumnDef, ColumnType, id_column, not_null_col};

export_table_names_for_kotlin!(
    pub const INCOMING_LOVE_LETTERS: &str = "incoming_love_letters",
);

export_column_names_for_kotlin!(
    incoming_love_letters,
    pub const LOVE_LETTER: &str = "love_letter",
    pub const TARGET_PAGE_ID: &str = "target_page_id",
    pub const TIMESTAMP: &str = "timestamp",
    pub const APPLIED: &str = "applied",
    pub const SESSION_ID: &str = "session_id",
);

#[uniffi::export]
pub fn incoming_love_letters_columns() -> Vec<ColumnDef> {
    vec![
        id_column(),
        not_null_col(ColumnType::Blob, LOVE_LETTER),
        not_null_col(ColumnType::Text, TARGET_PAGE_ID),
        not_null_col(ColumnType::Integer, TIMESTAMP),
        not_null_col(ColumnType::Text, APPLIED),
        not_null_col(ColumnType::Text, SESSION_ID),
    ]
}
