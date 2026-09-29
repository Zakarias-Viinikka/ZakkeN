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

pub enum ColumnsLogs {
    Timestamp,
    Level,
    Category,
    Source,
    SessionId,
    Message,
    Details,
    DetailsType,
}

pub fn get_table_name_logs() -> String {
    "logs".into()
}

const TIMESTAMP: SchemaColumn<i64> = SchemaColumn::<i64> {
    name: "timestamp",
    type_of_col: &TypeOfCol::Integer,
    _marker: PhantomData,
};

const LEVEL: SchemaColumn<String> = SchemaColumn::<String> {
    name: "level",
    type_of_col: &TypeOfCol::Text,
    _marker: PhantomData,
};

const CATEGORY: SchemaColumn<String> = SchemaColumn::<String> {
    name: "category",
    type_of_col: &TypeOfCol::Text,
    _marker: PhantomData,
};

const SOURCE: SchemaColumn<String> = SchemaColumn::<String> {
    name: "source",
    type_of_col: &TypeOfCol::Text,
    _marker: PhantomData,
};

const SESSION_ID: SchemaColumn<String> = SchemaColumn::<String> {
    name: "session_id",
    type_of_col: &TypeOfCol::Text,
    _marker: PhantomData,
};

const MESSAGE: SchemaColumn<String> = SchemaColumn::<String> {
    name: "message",
    type_of_col: &TypeOfCol::Text,
    _marker: PhantomData,
};

const DETAILS: SchemaColumn<Vec<u8>> = SchemaColumn::<Vec<u8>> {
    name: "details",
    type_of_col: &TypeOfCol::Blob,
    _marker: PhantomData,
};

const DETAILS_TYPE: SchemaColumn<String> = SchemaColumn::<String> {
    name: "details_type",
    type_of_col: &TypeOfCol::Text,
    _marker: PhantomData,
};

pub fn get_column_name(ENUM: ColumnsLogs) -> String {
    match ENUM {
        ColumnsLogs::Timestamp => TIMESTAMP.name.to_string(),
        ColumnsLogs::Level => LEVEL.name.to_string(),
        ColumnsLogs::Category => CATEGORY.name.to_string(),
        ColumnsLogs::Source => SOURCE.name.to_string(),
        ColumnsLogs::SessionId => SESSION_ID.name.to_string(),
        ColumnsLogs::Message => MESSAGE.name.to_string(),
        ColumnsLogs::Details => DETAILS.name.to_string(),
        ColumnsLogs::DetailsType => DETAILS_TYPE.name.to_string(),
    }
}

pub fn destruct_col_logs_timestamp(
    col_to_destruct: protocol::row_col::Col,
) -> Result<i64, DbError> {
    TIMESTAMP.destruct_db_col(col_to_destruct).map_err(|_| {
        ColDestructFail(format!(
            "Failed  to destruct to type: {:?}",
            TIMESTAMP.type_of_col
        ))
    })
}

pub fn destruct_col_logs_level(col_to_destruct: protocol::row_col::Col) -> Result<String, DbError> {
    LEVEL.destruct_db_col(col_to_destruct).map_err(|_| {
        ColDestructFail(format!(
            "Failed  to destruct to type: {:?}",
            LEVEL.type_of_col
        ))
    })
}

pub fn destruct_col_logs_category(
    col_to_destruct: protocol::row_col::Col,
) -> Result<String, DbError> {
    CATEGORY.destruct_db_col(col_to_destruct).map_err(|_| {
        ColDestructFail(format!(
            "Failed  to destruct to type: {:?}",
            CATEGORY.type_of_col
        ))
    })
}

pub fn destruct_col_logs_source(
    col_to_destruct: protocol::row_col::Col,
) -> Result<String, DbError> {
    SOURCE.destruct_db_col(col_to_destruct).map_err(|_| {
        ColDestructFail(format!(
            "Failed  to destruct to type: {:?}",
            SOURCE.type_of_col
        ))
    })
}

pub fn destruct_col_logs_session_id(
    col_to_destruct: protocol::row_col::Col,
) -> Result<String, DbError> {
    SESSION_ID.destruct_db_col(col_to_destruct).map_err(|_| {
        ColDestructFail(format!(
            "Failed  to destruct to type: {:?}",
            SESSION_ID.type_of_col
        ))
    })
}

pub fn destruct_col_logs_message(
    col_to_destruct: protocol::row_col::Col,
) -> Result<String, DbError> {
    MESSAGE.destruct_db_col(col_to_destruct).map_err(|_| {
        ColDestructFail(format!(
            "Failed  to destruct to type: {:?}",
            MESSAGE.type_of_col
        ))
    })
}

pub fn destruct_col_logs_details(
    col_to_destruct: protocol::row_col::Col,
) -> Result<Vec<u8>, DbError> {
    DETAILS.destruct_db_col(col_to_destruct).map_err(|_| {
        ColDestructFail(format!(
            "Failed  to destruct to type: {:?}",
            DETAILS.type_of_col
        ))
    })
}

pub fn destruct_col_logs_details_type(
    col_to_destruct: protocol::row_col::Col,
) -> Result<String, DbError> {
    DETAILS_TYPE.destruct_db_col(col_to_destruct).map_err(|_| {
        ColDestructFail(format!(
            "Failed  to destruct to type: {:?}",
            DETAILS_TYPE.type_of_col
        ))
    })
}
