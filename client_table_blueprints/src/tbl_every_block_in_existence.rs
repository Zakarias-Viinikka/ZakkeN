use crate::export_column_names_for_kotlin;
use crate::export_table_names_for_kotlin;
use protocol::new_table::{ColumnDef, ColumnType, ForeignKeyDef, id_column, not_null_col};

export_table_names_for_kotlin!(
    pub const EVERY_BLOCK_IN_EXISTENCE: &str = "every_block_in_existence",
);

export_column_names_for_kotlin!(
    every_block_in_existence,
    pub const IS_TITLE: &str = "is_title",
    pub const IS_PART_OF_MAIN_MENU_PAGE: &str = "is_part_of_main_menu_page",
    pub const CONTENT: &str = "content",
    pub const MY_ID_AS_GIVEN_BY_YRS: &str = "my_id_as_given_by_yrs",
    pub const ID_OF_PAGE_I_BELONG_TO: &str = "id_of_page_i_belong_to",
    pub const POSITION: &str = "position",
);

#[uniffi::export]
pub fn every_block_in_existence_columns() -> Vec<ColumnDef> {
    vec![
        id_column(),
        not_null_col(ColumnType::Text, IS_TITLE),
        not_null_col(ColumnType::Text, IS_PART_OF_MAIN_MENU_PAGE),
        not_null_col(ColumnType::Text, CONTENT),
        not_null_col(ColumnType::Text, MY_ID_AS_GIVEN_BY_YRS),
        not_null_col(ColumnType::Text, ID_OF_PAGE_I_BELONG_TO),
        not_null_col(ColumnType::Real, POSITION),
    ]
}

#[uniffi::export]
pub fn get_foreign_def_every_block_in_existence() -> Vec<ForeignKeyDef> {
    vec![ForeignKeyDef {
        column: ID_OF_PAGE_I_BELONG_TO.to_string(),
        referenced_table: crate::tbl_pages::PAGES.to_string(),
        referenced_column: crate::tbl_pages::PAGE_ID.to_string(),
    }]
}
