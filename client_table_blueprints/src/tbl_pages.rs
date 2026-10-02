#![allow(non_snake_case)]
use std::marker::PhantomData;

//use protocol::error::DbError;
//use protocol::error::DbError::ColDestructFail;
//use protocol::schema_helper::DestructDbReturnCol;
use protocol::new_table::ColumnDef;
use protocol::new_table::ColumnType;
use protocol::new_table::id_column;
use protocol::new_table::not_null_col;
use protocol::new_table::not_null_unique_col;
use protocol::schema_helper::{SchemaColumn, TypeOfCol};

#[uniffi::export]
pub fn new_table_pages() -> Vec<ColumnDef> {
    vec![
        id_column(),
        not_null_unique_col(ColumnType::Text, "page_id"),
        not_null_col(ColumnType::Blob, "blobbed_page"),
        not_null_col(ColumnType::Blob, "page_status"),
        not_null_col(ColumnType::Blob, "version"),
        not_null_col(ColumnType::Text, "is_main_menu_page"),
    ]
}

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

// ---
/*
   new row helper here
*/
// ---
use std::sync::Arc;

use my_yrs_lib::{YrsActivePages, YrsError};
use protocol::payload::ColumnValue;
use protocol::row_col::Col;

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
        ColumnValue {
            column_name: PAGE_ID.name.to_string(),
            value: Col::Text(page_id),
        },
        ColumnValue {
            column_name: BLOBBED_PAGE.name.to_string(),
            value: Col::Blob(blobbed_page),
        },
        ColumnValue {
            column_name: PAGE_STATUS.name.to_string(),
            value: Col::Blob(page_status),
        },
        ColumnValue {
            column_name: VERSION.name.to_string(),
            value: Col::Blob(version),
        },
        ColumnValue {
            column_name: IS_MAIN_MENU_PAGE.name.to_string(),
            value: Col::Text(is_main_menu_page.to_string()),
        },
    ])
}
