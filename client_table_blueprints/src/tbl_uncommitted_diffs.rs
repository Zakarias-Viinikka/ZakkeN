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
pub fn new_table_uncommitted_diffs() -> Vec<ColumnDef> {
    vec![
        id_column(),
        not_null_col(ColumnType::Blob, "snapshot_of_edit"),
        not_null_col(ColumnType::Blob, "love_letter_sketch"),
        not_null_col(ColumnType::Text, "session_id"),
        not_null_col(ColumnType::Text, "target_id"),
    ]
}

pub enum ColumnsUncommittedDiffs {
    SnapshotOfEdit,
    LoveLetterSketch,
    SessionId,
    TargetId,
}

pub fn get_table_name_uncommitted_diffs() -> String {
    "uncommitted_diffs".into()
}

const SNAPSHOT_OF_EDIT: SchemaColumn<Vec<u8>> = SchemaColumn::<Vec<u8>> {
    name: "snapshot_of_edit",
    type_of_col: &TypeOfCol::Blob,
    can_be_null: false,
    _marker: PhantomData,
};

const LOVE_LETTER_SKETCH: SchemaColumn<Vec<u8>> = SchemaColumn::<Vec<u8>> {
    name: "love_letter_sketch",
    type_of_col: &TypeOfCol::Blob,
    can_be_null: false,
    _marker: PhantomData,
};

const SESSION_ID: SchemaColumn<String> = SchemaColumn::<String> {
    name: "session_id",
    type_of_col: &TypeOfCol::Text,
    can_be_null: false,
    _marker: PhantomData,
};

const TARGET_ID: SchemaColumn<String> = SchemaColumn::<String> {
    name: "target_id",
    type_of_col: &TypeOfCol::Text,
    can_be_null: false,
    _marker: PhantomData,
};

pub fn get_column_name(ENUM: ColumnsUncommittedDiffs) -> String {
    match ENUM {
        ColumnsUncommittedDiffs::SnapshotOfEdit => SNAPSHOT_OF_EDIT.name.to_string(),
        ColumnsUncommittedDiffs::LoveLetterSketch => LOVE_LETTER_SKETCH.name.to_string(),
        ColumnsUncommittedDiffs::SessionId => SESSION_ID.name.to_string(),
        ColumnsUncommittedDiffs::TargetId => TARGET_ID.name.to_string(),
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
pub fn new_uncommitted_diff_row(
    snapshot_of_edit: Vec<u8>,
    love_letter_sketch: Vec<u8>,
    session_id: String,
    target_id: String,
) -> Result<Vec<ColumnValue>, YrsError> {
    Ok(vec![
        ColumnValue {
            column_name: SNAPSHOT_OF_EDIT.name.to_string(),
            value: Col::Blob(snapshot_of_edit),
        },
        ColumnValue {
            column_name: LOVE_LETTER_SKETCH.name.to_string(),
            value: Col::Blob(love_letter_sketch),
        },
        ColumnValue {
            column_name: SESSION_ID.name.to_string(),
            value: Col::Text(session_id),
        },
        ColumnValue {
            column_name: TARGET_ID.name.to_string(),
            value: Col::Text(target_id),
        },
    ])
}
