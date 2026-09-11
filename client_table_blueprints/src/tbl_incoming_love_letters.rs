use protocol::new_table::{ColumnDef, ColumnType, id_column, not_null_col};

#[uniffi::export]
pub fn incoming_love_letters_columns() -> Vec<ColumnDef> {
    vec![
        id_column(),
        not_null_col(ColumnType::Blob, "love_letter"),
        not_null_col(ColumnType::Text, "target_page_id"),
        not_null_col(ColumnType::Integer, "timestamp"),
        not_null_col(ColumnType::Text, "applied"),
        not_null_col(ColumnType::Text, "session_id"),
    ]
}
