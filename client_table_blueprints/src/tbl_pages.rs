use std::marker::PhantomData;
use std::sync::Arc;

use my_yrs_lib::{YrsActivePages, YrsError};
use protocol::new_table::ColumnDef;
use protocol::new_table::ColumnType;
use protocol::new_table::id_column;
use protocol::new_table::not_null_col;
use protocol::new_table::not_null_unique_col;
use protocol::payload::ColumnValue;
use protocol::schema_helper::{SchemaColumn, TypeOfCol};

use crate::row_values::{blob_value, text_value};

pub const TABLE_NAME: &str = "pages";

pub const PAGE_ID: SchemaColumn<String> = SchemaColumn::<String> {
    name: "page_id",
    type_of_col: &TypeOfCol::Text,
    can_be_null: false,
    _marker: PhantomData,
};

pub const BLOBBED_PAGE: SchemaColumn<Vec<u8>> = SchemaColumn::<Vec<u8>> {
    name: "blobbed_page",
    type_of_col: &TypeOfCol::Blob,
    can_be_null: false,
    _marker: PhantomData,
};

pub const PAGE_STATUS: SchemaColumn<Vec<u8>> = SchemaColumn::<Vec<u8>> {
    name: "page_status",
    type_of_col: &TypeOfCol::Blob,
    can_be_null: false,
    _marker: PhantomData,
};

pub const VERSION: SchemaColumn<Vec<u8>> = SchemaColumn::<Vec<u8>> {
    name: "version",
    type_of_col: &TypeOfCol::Blob,
    can_be_null: false,
    _marker: PhantomData,
};

pub const IS_MAIN_MENU_PAGE: SchemaColumn<String> = SchemaColumn::<String> {
    name: "is_main_menu_page",
    type_of_col: &TypeOfCol::Text,
    can_be_null: false,
    _marker: PhantomData,
};

#[uniffi::export]
pub fn new_table_pages() -> Vec<ColumnDef> {
    vec![
        id_column(),
        not_null_unique_col(ColumnType::Text, PAGE_ID.name),
        not_null_col(ColumnType::Blob, BLOBBED_PAGE.name),
        not_null_col(ColumnType::Blob, PAGE_STATUS.name),
        not_null_col(ColumnType::Blob, VERSION.name),
        not_null_col(ColumnType::Text, IS_MAIN_MENU_PAGE.name),
    ]
}

#[uniffi::export]
pub fn new_page_row(
    page_id: String,
    is_main_menu_page: bool,
    blobbed_page: Vec<u8>,
    version: Vec<u8>,
) -> Result<Vec<ColumnValue>, YrsError> {
    let active_doc = Arc::new(YrsActivePages::new_empty());
    let page_status = active_doc.snapshot()?;

    Ok(vec![
        text_value(&PAGE_ID, page_id),
        blob_value(&BLOBBED_PAGE, blobbed_page),
        blob_value(&PAGE_STATUS, page_status),
        blob_value(&VERSION, version),
        text_value(&IS_MAIN_MENU_PAGE, is_main_menu_page.to_string()),
    ])
}