use std::marker::PhantomData;
use std::sync::Arc;

use my_yrs_lib::YrsBacklinks;
use my_yrs_lib::YrsError;
use protocol::new_table::ColumnDef;
use protocol::new_table::ColumnType;
use protocol::new_table::ForeignKeyDef;
use protocol::new_table::id_column;
use protocol::new_table::not_null_col;
use protocol::payload::ColumnValue;
use protocol::schema_helper::{SchemaColumn, TypeOfCol};

use crate::row_values::{blob_value, text_value};

pub const TABLE_NAME: &str = "backlinks";

pub const PAGE_THAT_HOLDS_LINK_ID: SchemaColumn<String> = SchemaColumn::<String> {
    name: "page_that_holds_link_id",
    type_of_col: &TypeOfCol::Text,
    can_be_null: false,
    _marker: PhantomData,
};

pub const PAGE_BEING_LINKED_TO_ID: SchemaColumn<String> = SchemaColumn::<String> {
    name: "page_being_linked_to_id",
    type_of_col: &TypeOfCol::Text,
    can_be_null: false,
    _marker: PhantomData,
};

pub const DISABLED: SchemaColumn<Vec<u8>> = SchemaColumn::<Vec<u8>> {
    name: "disabled",
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

#[uniffi::export]
pub fn new_table_backlinks() -> Vec<ColumnDef> {
    vec![
        id_column(),
        not_null_col(ColumnType::Text, PAGE_THAT_HOLDS_LINK_ID.name),
        not_null_col(ColumnType::Text, PAGE_BEING_LINKED_TO_ID.name),
        not_null_col(ColumnType::Blob, DISABLED.name),
        not_null_col(ColumnType::Blob, VERSION.name),
    ]
}

#[uniffi::export]
pub fn get_foreign_def_backlinks() -> Vec<ForeignKeyDef> {
    vec![
        ForeignKeyDef {
            column_name: PAGE_THAT_HOLDS_LINK_ID.name.to_string(),
            referenced_table_name: crate::tbl_pages::TABLE_NAME.to_string(),
            referenced_column_name: crate::tbl_pages::PAGE_ID.name.to_string(),
        },
        ForeignKeyDef {
            column_name: PAGE_BEING_LINKED_TO_ID.name.to_string(),
            referenced_table_name: crate::tbl_pages::TABLE_NAME.to_string(),
            referenced_column_name: crate::tbl_pages::PAGE_ID.name.to_string(),
        },
    ]
}

#[uniffi::export]
pub fn new_backlink_row(
    page_that_holds_link_id: String,
    page_being_linked_to_id: String,
) -> Result<Vec<ColumnValue>, YrsError> {
    let backlinks_doc = Arc::new(YrsBacklinks::new_empty());
    let disabled = backlinks_doc.clone().snapshot()?;
    let version = backlinks_doc.create_bookmark()?;

    Ok(vec![
        text_value(&PAGE_THAT_HOLDS_LINK_ID, page_that_holds_link_id),
        text_value(&PAGE_BEING_LINKED_TO_ID, page_being_linked_to_id),
        blob_value(&DISABLED, disabled),
        blob_value(&VERSION, version),
    ])
}