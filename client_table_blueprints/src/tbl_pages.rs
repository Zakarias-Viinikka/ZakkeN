use crate::export_column_names_for_kotlin;
use crate::export_table_names_for_kotlin;
use protocol::new_table::{ColumnDef, ColumnType, id_column, not_null_col, not_null_unique_col};

export_table_names_for_kotlin!(
    pub const PAGES: &str = "pages",
);

export_column_names_for_kotlin!(
    pages,
    pub const PAGE_ID: &str = "page_id",
    pub const BLOBBED_PAGE: &str = "blobbed_page",
    pub const PAGE_STATUS: &str = "page_status",
    pub const VERSION: &str = "version",
    pub const IS_MAIN_MENU_PAGE: &str = "is_main_menu_page",
);

#[uniffi::export]
pub fn pages_columns() -> Vec<ColumnDef> {
    vec![
        id_column(),
        not_null_unique_col(ColumnType::Text, PAGE_ID),
        not_null_col(ColumnType::Blob, BLOBBED_PAGE),
        not_null_col(ColumnType::Blob, PAGE_STATUS),
        not_null_col(ColumnType::Blob, VERSION),
        not_null_col(ColumnType::Text, IS_MAIN_MENU_PAGE),
    ]
}
