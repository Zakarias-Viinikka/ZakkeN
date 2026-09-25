use crate::export_column_names_for_kotlin;
use crate::export_table_names_for_kotlin;
use protocol::new_table::{ColumnDef, ColumnType, default_col, id_column, not_null_col};

export_table_names_for_kotlin!(
    pub const LOGS: &str = "logs",
);

export_column_names_for_kotlin!(
    logs,
    pub const TIMESTAMP: &str = "timestamp",
    pub const LEVEL: &str = "level",
    pub const CATEGORY: &str = "category",
    pub const SOURCE: &str = "source",
    pub const SESSION_ID: &str = "session_id",
    pub const MESSAGE: &str = "message",
    pub const DETAILS: &str = "details",
    pub const DETAILS_TYPE: &str = "details_type",
);

#[uniffi::export]
pub fn logs_columns() -> Vec<ColumnDef> {
    vec![
        id_column(),
        not_null_col(ColumnType::Integer, TIMESTAMP),
        not_null_col(ColumnType::Text, LEVEL),
        not_null_col(ColumnType::Text, CATEGORY),
        not_null_col(ColumnType::Text, SOURCE),
        not_null_col(ColumnType::Text, SESSION_ID),
        not_null_col(ColumnType::Text, MESSAGE),
        default_col(ColumnType::Blob, DETAILS),
        default_col(ColumnType::Text, DETAILS_TYPE),
    ]
}
