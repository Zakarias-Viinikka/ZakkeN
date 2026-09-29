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
    _marker: PhantomData,
};

const LOVE_LETTER_SKETCH: SchemaColumn<Vec<u8>> = SchemaColumn::<Vec<u8>> {
    name: "love_letter_sketch",
    type_of_col: &TypeOfCol::Blob,
    _marker: PhantomData,
};

const SESSION_ID: SchemaColumn<String> = SchemaColumn::<String> {
    name: "session_id",
    type_of_col: &TypeOfCol::Text,
    _marker: PhantomData,
};

const TARGET_ID: SchemaColumn<String> = SchemaColumn::<String> {
    name: "target_id",
    type_of_col: &TypeOfCol::Text,
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

pub fn destruct_col_uncommitted_diffs_snapshot_of_edit(
    col_to_destruct: protocol::row_col::Col,
) -> Result<Vec<u8>, DbError> {
    SNAPSHOT_OF_EDIT
        .destruct_db_col(col_to_destruct)
        .map_err(|_| {
            ColDestructFail(format!(
                "Failed  to destruct to type: {:?}",
                SNAPSHOT_OF_EDIT.type_of_col
            ))
        })
}

pub fn destruct_col_uncommitted_diffs_love_letter_sketch(
    col_to_destruct: protocol::row_col::Col,
) -> Result<Vec<u8>, DbError> {
    LOVE_LETTER_SKETCH
        .destruct_db_col(col_to_destruct)
        .map_err(|_| {
            ColDestructFail(format!(
                "Failed  to destruct to type: {:?}",
                LOVE_LETTER_SKETCH.type_of_col
            ))
        })
}

pub fn destruct_col_uncommitted_diffs_session_id(
    col_to_destruct: protocol::row_col::Col,
) -> Result<String, DbError> {
    SESSION_ID.destruct_db_col(col_to_destruct).map_err(|_| {
        ColDestructFail(format!(
            "Failed  to destruct to type: {:?}",
            SESSION_ID.type_of_col
        ))
    })
}

pub fn destruct_col_uncommitted_diffs_target_id(
    col_to_destruct: protocol::row_col::Col,
) -> Result<String, DbError> {
    TARGET_ID.destruct_db_col(col_to_destruct).map_err(|_| {
        ColDestructFail(format!(
            "Failed  to destruct to type: {:?}",
            TARGET_ID.type_of_col
        ))
    })
}
