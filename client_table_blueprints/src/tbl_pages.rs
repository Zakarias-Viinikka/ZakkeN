#![allow(non_snake_case)]
use std::marker::PhantomData;

use protocol::error::DbError;
use protocol::error::DbError::ColDestructFail;
use protocol::schema_helper::DestructDbReturnCol;
use protocol::schema_helper::{SchemaColumn, TypeOfCol};

// enum for columns
// method for destructing that takes the enum + col to destruct
// get_colum_name that the enum "points" to
// get_type cuz why not? might be useful

pub enum ColumnsPages {
    PageId,
    BlobbedPage,
    PageStatus,
    Version,
    IsMainMenuPage,
}

pub fn get_table_name_pages() -> String {
    "pages".into()
}

pub const PAGE_ID: SchemaColumn<String> = SchemaColumn::<String> {
    name: "page_id",
    type_of_col: &TypeOfCol::Text,
    _marker: PhantomData,
};

const BLOBBED_PAGE: SchemaColumn<Vec<u8>> = SchemaColumn::<Vec<u8>> {
    name: "blobbed_page",
    type_of_col: &TypeOfCol::Blob,
    _marker: PhantomData,
};

const PAGE_STATUS: SchemaColumn<Vec<u8>> = SchemaColumn::<Vec<u8>> {
    name: "page_status",
    type_of_col: &TypeOfCol::Blob,
    _marker: PhantomData,
};

const VERSION: SchemaColumn<Vec<u8>> = SchemaColumn::<Vec<u8>> {
    name: "version",
    type_of_col: &TypeOfCol::Blob,
    _marker: PhantomData,
};

const IS_MAIN_MENU_PAGE: SchemaColumn<String> = SchemaColumn::<String> {
    name: "is_main_menu_page",
    type_of_col: &TypeOfCol::Text,
    _marker: PhantomData,
};

pub fn get_column_name(ENUM: ColumnsPages) -> String {
    match ENUM {
        ColumnsPages::PageId => PAGE_ID.name.to_string(),
        ColumnsPages::BlobbedPage => BLOBBED_PAGE.name.to_string(),
        ColumnsPages::PageStatus => PAGE_STATUS.name.to_string(),
        ColumnsPages::Version => VERSION.name.to_string(),
        ColumnsPages::IsMainMenuPage => IS_MAIN_MENU_PAGE.name.to_string(),
    }
}

pub fn destruct_col_pages_page_id(
    col_to_destruct: protocol::row_col::Col,
) -> Result<String, DbError> {
    PAGE_ID.destruct_db_col(col_to_destruct).map_err(|_| {
        ColDestructFail(format!(
            "Failed  to destruct to type: {:?}",
            PAGE_ID.type_of_col
        ))
    })
}

pub fn destruct_col_pages_blobbed_page(
    col_to_destruct: protocol::row_col::Col,
) -> Result<Vec<u8>, DbError> {
    BLOBBED_PAGE.destruct_db_col(col_to_destruct).map_err(|_| {
        ColDestructFail(format!(
            "Failed  to destruct to type: {:?}",
            BLOBBED_PAGE.type_of_col
        ))
    })
}

pub fn destruct_col_pages_page_status(
    col_to_destruct: protocol::row_col::Col,
) -> Result<Vec<u8>, DbError> {
    PAGE_STATUS.destruct_db_col(col_to_destruct).map_err(|_| {
        ColDestructFail(format!(
            "Failed  to destruct to type: {:?}",
            PAGE_STATUS.type_of_col
        ))
    })
}

pub fn destruct_col_pages_version(
    col_to_destruct: protocol::row_col::Col,
) -> Result<Vec<u8>, DbError> {
    VERSION.destruct_db_col(col_to_destruct).map_err(|_| {
        ColDestructFail(format!(
            "Failed  to destruct to type: {:?}",
            VERSION.type_of_col
        ))
    })
}

pub fn destruct_col_pages_is_main_menu_page(
    col_to_destruct: protocol::row_col::Col,
) -> Result<String, DbError> {
    IS_MAIN_MENU_PAGE
        .destruct_db_col(col_to_destruct)
        .map_err(|_| {
            ColDestructFail(format!(
                "Failed  to destruct to type: {:?}",
                IS_MAIN_MENU_PAGE.type_of_col
            ))
        })
}
