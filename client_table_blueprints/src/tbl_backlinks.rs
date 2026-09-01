use protocol::new_table::{ColumnDef, ColumnType, ForeignKeyDef, id_column, not_null_col};

#[uniffi::export]
pub fn backlinks_columns() -> Vec<ColumnDef> {
    vec![
        id_column(),
        not_null_col(ColumnType::Text, "page_that_holds_link_id"),
        not_null_col(ColumnType::Text, "page_being_linked_to_id"),
        not_null_col(ColumnType::Blob, "disabled"),
        not_null_col(ColumnType::Blob, "version"),
    ]
}

#[uniffi::export]
pub fn get_foreign_def_backlinks() -> Vec<ForeignKeyDef> {
    vec![
        ForeignKeyDef {
            column: "page_that_holds_link_id".to_string(),
            referenced_table: "pages".to_string(),
            referenced_column: "page_id".to_string(),
        },
        ForeignKeyDef {
            column: "page_being_linked_to_id".to_string(),
            referenced_table: "pages".to_string(),
            referenced_column: "page_id".to_string(),
        },
    ]
}
