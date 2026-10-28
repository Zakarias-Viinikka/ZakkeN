use std::marker::PhantomData;

use my_yrs_lib::YrsError;
use protocol::new_table::ColumnDef;
use protocol::new_table::ColumnType;
use protocol::new_table::id_column;
use protocol::new_table::not_null_col;
use protocol::payload::ColumnValue;
use protocol::schema_helper::{SchemaColumn, TypeOfCol};

use crate::row_values::{blob_value, text_value};

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

#[uniffi::export]
pub fn new_table_incoming_love_letters() -> Vec<ColumnDef> {
    vec![
        id_column(),
        not_null_col(ColumnType::Blob, LOVE_LETTER.name),
        not_null_col(ColumnType::Text, TARGET_PAGE_ID.name),
    ]
}

#[uniffi::export]
pub fn new_incoming_love_letter_row(
    love_letter: Vec<u8>,
    target_page_id: String,
) -> Result<Vec<ColumnValue>, YrsError> {
    Ok(vec![
        blob_value(&LOVE_LETTER, love_letter),
        text_value(&TARGET_PAGE_ID, target_page_id),
    ])
}