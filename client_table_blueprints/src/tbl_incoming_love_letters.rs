#![allow(non_snake_case)]
use std::marker::PhantomData;

//use protocol::error::DbError;
//use protocol::error::DbError::ColDestructFail;
//use protocol::schema_helper::DestructDbReturnCol;
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
        not_null_col(ColumnType::Integer, "timestamp"),
        not_null_col(ColumnType::Text, "applied"),
        not_null_col(ColumnType::Text, "session_id"),
    ]
}

pub enum ColumnsIncomingLoveLetters {
    LoveLetter,
    TargetPageId,
    Timestamp,
    Applied,
    SessionId,
}

pub fn get_table_name_incoming_love_letters() -> String {
    "incoming_love_letters".into()
}

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

pub const TIMESTAMP: SchemaColumn<i64> = SchemaColumn::<i64> {
    name: "timestamp",
    type_of_col: &TypeOfCol::Integer,
    can_be_null: false,
    _marker: PhantomData,
};

pub const APPLIED: SchemaColumn<String> = SchemaColumn::<String> {
    name: "applied",
    type_of_col: &TypeOfCol::Text,
    can_be_null: false,
    _marker: PhantomData,
};

pub const SESSION_ID: SchemaColumn<String> = SchemaColumn::<String> {
    name: "session_id",
    type_of_col: &TypeOfCol::Text,
    can_be_null: false,
    _marker: PhantomData,
};

pub fn get_column_name(ENUM: ColumnsIncomingLoveLetters) -> String {
    match ENUM {
        ColumnsIncomingLoveLetters::LoveLetter => LOVE_LETTER.name.to_string(),
        ColumnsIncomingLoveLetters::TargetPageId => TARGET_PAGE_ID.name.to_string(),
        ColumnsIncomingLoveLetters::Timestamp => TIMESTAMP.name.to_string(),
        ColumnsIncomingLoveLetters::Applied => APPLIED.name.to_string(),
        ColumnsIncomingLoveLetters::SessionId => SESSION_ID.name.to_string(),
    }
}

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
    timestamp: i64, //i dont remember why i have this col. but ill leave it for now
    session_id: String,
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
        ColumnValue {
            column_name: TIMESTAMP.name.to_string(),
            value: Col::Integer(timestamp),
        },
        ColumnValue {
            column_name: APPLIED.name.to_string(),
            value: Col::Text("false".to_string()),
        },
        ColumnValue {
            column_name: SESSION_ID.name.to_string(),
            value: Col::Text(session_id),
        },
    ])
}
