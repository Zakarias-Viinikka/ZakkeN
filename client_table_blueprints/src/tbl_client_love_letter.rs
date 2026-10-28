use std::marker::PhantomData;

use my_yrs_lib::YrsError;
use protocol::new_table::ColumnDef;
use protocol::new_table::ColumnType;
use protocol::new_table::id_column;
use protocol::new_table::not_null_col;
use protocol::payload::ColumnValue;
use protocol::schema_helper::{SchemaColumn, TypeOfCol};

use crate::row_values::{blob_value, text_value};

pub const TABLE_NAME: &str = "client_love_letter";

pub const SNAPSHOT_OF_EDIT: SchemaColumn<Vec<u8>> = SchemaColumn::<Vec<u8>> {
    name: "snapshot_of_edit",
    type_of_col: &TypeOfCol::Blob,
    can_be_null: false,
    _marker: PhantomData,
};

pub const LOVE_LETTER_SKETCH: SchemaColumn<Vec<u8>> = SchemaColumn::<Vec<u8>> {
    name: "love_letter_sketch",
    type_of_col: &TypeOfCol::Blob,
    can_be_null: false,
    _marker: PhantomData,
};

pub const SESSION_ID: SchemaColumn<String> = SchemaColumn::<String> {
    name: "session_id",
    type_of_col: &TypeOfCol::Text,
    can_be_null: false,
    _marker: PhantomData,
};

pub const TARGET_ID: SchemaColumn<String> = SchemaColumn::<String> {
    name: "target_id",
    type_of_col: &TypeOfCol::Text,
    can_be_null: false,
    _marker: PhantomData,
};

#[uniffi::export]
pub fn new_table_client_love_letter() -> Vec<ColumnDef> {
    vec![
        id_column(),
        not_null_col(ColumnType::Blob, SNAPSHOT_OF_EDIT.name),
        not_null_col(ColumnType::Blob, LOVE_LETTER_SKETCH.name),
        not_null_col(ColumnType::Text, SESSION_ID.name),
        not_null_col(ColumnType::Text, TARGET_ID.name),
    ]
}

#[uniffi::export]
pub fn new_client_love_letter_row(
    snapshot_of_edit: Vec<u8>,
    love_letter_sketch: Vec<u8>,
    session_id: String,
    target_id: String,
) -> Result<Vec<ColumnValue>, YrsError> {
    Ok(vec![
        blob_value(&SNAPSHOT_OF_EDIT, snapshot_of_edit),
        blob_value(&LOVE_LETTER_SKETCH, love_letter_sketch),
        text_value(&SESSION_ID, session_id),
        text_value(&TARGET_ID, target_id),
    ])
}