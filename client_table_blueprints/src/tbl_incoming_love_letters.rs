#![allow(non_snake_case)]
use std::marker::PhantomData;

use protocol::error::DbError;
use protocol::error::DbError::ColDestructFail;
use protocol::schema_helper::DestructDbReturnCol;
use protocol::schema_helper::{SchemaColumn, TypeOfCol};

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

const LOVE_LETTER: SchemaColumn<Vec<u8>> = SchemaColumn::<Vec<u8>> {
    name: "love_letter",
    type_of_col: &TypeOfCol::Blob,
    _marker: PhantomData,
};

const TARGET_PAGE_ID: SchemaColumn<String> = SchemaColumn::<String> {
    name: "target_page_id",
    type_of_col: &TypeOfCol::Text,
    _marker: PhantomData,
};

const TIMESTAMP: SchemaColumn<i64> = SchemaColumn::<i64> {
    name: "timestamp",
    type_of_col: &TypeOfCol::Integer,
    _marker: PhantomData,
};

const APPLIED: SchemaColumn<String> = SchemaColumn::<String> {
    name: "applied",
    type_of_col: &TypeOfCol::Text,
    _marker: PhantomData,
};

const SESSION_ID: SchemaColumn<String> = SchemaColumn::<String> {
    name: "session_id",
    type_of_col: &TypeOfCol::Text,
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

pub fn destruct_col_incoming_love_letters_love_letter(
    col_to_destruct: protocol::row_col::Col,
) -> Result<Vec<u8>, DbError> {
    LOVE_LETTER.destruct_db_col(col_to_destruct).map_err(|_| {
        ColDestructFail(format!(
            "Failed  to destruct to type: {:?}",
            LOVE_LETTER.type_of_col
        ))
    })
}

pub fn destruct_col_incoming_love_letters_target_page_id(
    col_to_destruct: protocol::row_col::Col,
) -> Result<String, DbError> {
    TARGET_PAGE_ID
        .destruct_db_col(col_to_destruct)
        .map_err(|_| {
            ColDestructFail(format!(
                "Failed  to destruct to type: {:?}",
                TARGET_PAGE_ID.type_of_col
            ))
        })
}

pub fn destruct_col_incoming_love_letters_timestamp(
    col_to_destruct: protocol::row_col::Col,
) -> Result<i64, DbError> {
    TIMESTAMP.destruct_db_col(col_to_destruct).map_err(|_| {
        ColDestructFail(format!(
            "Failed  to destruct to type: {:?}",
            TIMESTAMP.type_of_col
        ))
    })
}

pub fn destruct_col_incoming_love_letters_applied(
    col_to_destruct: protocol::row_col::Col,
) -> Result<String, DbError> {
    APPLIED.destruct_db_col(col_to_destruct).map_err(|_| {
        ColDestructFail(format!(
            "Failed  to destruct to type: {:?}",
            APPLIED.type_of_col
        ))
    })
}

pub fn destruct_col_incoming_love_letters_session_id(
    col_to_destruct: protocol::row_col::Col,
) -> Result<String, DbError> {
    SESSION_ID.destruct_db_col(col_to_destruct).map_err(|_| {
        ColDestructFail(format!(
            "Failed  to destruct to type: {:?}",
            SESSION_ID.type_of_col
        ))
    })
}
