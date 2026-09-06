use protocol::new_table::{ColumnDef, ColumnType, ForeignKeyDef, id_column, not_null_col};

#[uniffi::export]
pub fn every_block_in_existence_columns() -> Vec<ColumnDef> {
    vec![
        id_column(),
        not_null_col(ColumnType::Text, "title"),
        not_null_col(ColumnType::Text, "page_that_owns_me"),
        not_null_col(ColumnType::Text, "content"),
        not_null_col(ColumnType::Text, "my_id_as_given_by_yrs"),
        not_null_col(ColumnType::Text, "id_of_page_i_belong_to"),
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
