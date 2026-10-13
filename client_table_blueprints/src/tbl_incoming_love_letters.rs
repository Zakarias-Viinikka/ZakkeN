#![allow(non_snake_case)]
use std::marker::PhantomData;

use protocol::new_table::ColumnDef;
use protocol::new_table::ColumnType;
use protocol::new_table::id_column;
use protocol::new_table::not_null_col;
use protocol::schema_helper::{SchemaColumn, TypeOfCol};

#[uniffi::export]
pub fn new_table_incoming_love_letters() -> Vec<ColumnDef> {
    vec![
        id_column(),
        not_null_col(ColumnType::Blob, "love_letter"),
        not_null_col(ColumnType::Text, "target_page_id"),
    ]
}

pub const TABLE_NAME: &str = "incoming_love_letters";

pub const LOVE_LETTER: SchemaColumn<Vec<u8>> = SchemaColumn::<Vec<u8>> {
    name: "love_letter",
    type_of_col: &TypeOfCol::Blob,
    can_be_null: false,
    _marker: PhantomData,
};

pub const TARGET_PAGE_ID: SchemaColumn<String> = SchemaColumn::<String> {
    name: "target_page_id",
    type_of_col: &TypeOfCol::Text,
    can_be_null: false,
    _marker: PhantomData,
};

// ---
/*
   new row helper here
*/
// ---
use my_yrs_lib::YrsError;
use protocol::payload::ColumnValue;
use protocol::row_col::Col;

#[uniffi::export]
pub fn new_incoming_love_letter_row(
    love_letter: Vec<u8>,
    target_page_id: String,
) -> Result<Vec<ColumnValue>, YrsError> {
    Ok(vec![
        ColumnValue {
            column_name: LOVE_LETTER.name.to_string(),
            value: Col::Blob(love_letter),
        },
        ColumnValue {
            column_name: TARGET_PAGE_ID.name.to_string(),
            value: Col::Text(target_page_id),
        },
    ])
}