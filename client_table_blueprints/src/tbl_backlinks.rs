use std::sync::Arc;

use crate::export_column_names_for_kotlin;
use crate::export_table_names_for_kotlin;
use my_yrs_lib::YrsBacklinks;
use my_yrs_lib::YrsError;
use protocol::new_table::{ColumnDef, ColumnType, ForeignKeyDef, id_column, not_null_col};
use protocol::payload::ColumnValue;
use protocol::row_col::Col;

export_table_names_for_kotlin!(
    pub const BACKLINKS: &str = "backlinks",
);

export_column_names_for_kotlin!(
    backlinks,
    pub const PAGE_THAT_HOLDS_LINK_ID: &str = "page_that_holds_link_id",
    pub const PAGE_BEING_LINKED_TO_ID: &str = "page_being_linked_to_id",
    pub const DISABLED: &str = "disabled",
    pub const VERSION: &str = "version",
);

#[uniffi::export]
pub fn new_backlink_row(
    page_that_holds_link_id: String,
    page_being_linked_to_id: String,
) -> Result<Vec<ColumnValue>, YrsError> {
    let backlinks_doc = Arc::new(YrsBacklinks::new_empty());
    let disabled = backlinks_doc.clone().snapshot()?;
    let version = backlinks_doc.create_bookmark_of_synced_state()?;

    Ok(vec![
        ColumnValue {
            column_name: PAGE_THAT_HOLDS_LINK_ID.to_string(),
            value: Col::Text(page_that_holds_link_id),
        },
        ColumnValue {
            column_name: PAGE_BEING_LINKED_TO_ID.to_string(),
            value: Col::Text(page_being_linked_to_id),
        },
        ColumnValue {
            column_name: DISABLED.to_string(),
            value: Col::Blob(disabled),
        },
        ColumnValue {
            column_name: VERSION.to_string(),
            value: Col::Blob(version),
        },
    ])
}

#[uniffi::export]
pub fn backlinks_columns() -> Vec<ColumnDef> {
    vec![
        id_column(),
        not_null_col(ColumnType::Text, PAGE_THAT_HOLDS_LINK_ID),
        not_null_col(ColumnType::Text, PAGE_BEING_LINKED_TO_ID),
        not_null_col(ColumnType::Blob, DISABLED),
        not_null_col(ColumnType::Blob, VERSION),
    ]
}

#[uniffi::export]
pub fn get_foreign_def_backlinks() -> Vec<ForeignKeyDef> {
    vec![
        ForeignKeyDef {
            column: PAGE_THAT_HOLDS_LINK_ID.to_string(),
            referenced_table: crate::tbl_pages::PAGES.to_string(),
            referenced_column: crate::tbl_pages::PAGE_ID.to_string(),
        },
        ForeignKeyDef {
            column: PAGE_BEING_LINKED_TO_ID.to_string(),
            referenced_table: crate::tbl_pages::PAGES.to_string(),
            referenced_column: crate::tbl_pages::PAGE_ID.to_string(),
        },
    ]
}
