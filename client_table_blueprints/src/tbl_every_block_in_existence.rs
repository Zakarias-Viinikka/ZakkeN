use protocol::new_table::{ColumnDef, ColumnType, ForeignKeyDef, id_column, not_null_col};

#[uniffi::export]
pub fn every_block_in_existence_columns() -> Vec<ColumnDef> {
    vec![
        id_column(),
        not_null_col(ColumnType::Text, "page_that_owns_me"),
        not_null_col(ColumnType::Text, "content"),
        not_null_col(ColumnType::Text, "id_of_block_that_owns"),
    ]
}

#[uniffi::export]
pub fn get_foreign_def_every_block_in_existence() -> Vec<ForeignKeyDef> {
    vec![ForeignKeyDef {
        column: "page_that_owns_me".to_string(),
        referenced_table: "pages".to_string(),
        referenced_column: "page_id".to_string(),
    }]
}
