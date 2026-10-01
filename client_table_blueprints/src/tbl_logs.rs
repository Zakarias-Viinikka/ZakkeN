#![allow(non_snake_case)]
use std::marker::PhantomData;

//use protocol::error::DbError;
//use protocol::error::DbError::ColDestructFail;
//use protocol::schema_helper::DestructDbReturnCol;
use protocol::new_table::ColumnDef;
use protocol::new_table::ColumnType;
use protocol::new_table::default_col;
use protocol::new_table::id_column;
use protocol::new_table::not_null_col;
use protocol::row_col::StructRepresentingNull;
use protocol::schema_helper::{SchemaColumn, TypeOfCol};

#[uniffi::export]
pub fn new_table_logs() -> Vec<ColumnDef> {
    vec![
        id_column(),
        not_null_col(ColumnType::Integer, "timestamp"),
        not_null_col(ColumnType::Text, "level"),
        not_null_col(ColumnType::Text, "category"),
        not_null_col(ColumnType::Text, "source"),
        not_null_col(ColumnType::Text, "session_id"),
        not_null_col(ColumnType::Text, "message"),
        default_col(ColumnType::Blob, "details"),
        default_col(ColumnType::Text, "details_type"),
    ]
}

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

pub const TIMESTAMP: SchemaColumn<i64> = SchemaColumn::<i64> {
    name: "timestamp",
    type_of_col: &TypeOfCol::Integer,
    can_be_null: false,
    _marker: PhantomData,
};

pub const LEVEL: SchemaColumn<String> = SchemaColumn::<String> {
    name: "level",
    type_of_col: &TypeOfCol::Text,
    can_be_null: false,
    _marker: PhantomData,
};

pub const CATEGORY: SchemaColumn<String> = SchemaColumn::<String> {
    name: "category",
    type_of_col: &TypeOfCol::Text,
    can_be_null: false,
    _marker: PhantomData,
};

pub const SOURCE: SchemaColumn<String> = SchemaColumn::<String> {
    name: "source",
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

pub const MESSAGE: SchemaColumn<String> = SchemaColumn::<String> {
    name: "message",
    type_of_col: &TypeOfCol::Text,
    can_be_null: false,
    _marker: PhantomData,
};

pub const DETAILS: SchemaColumn<Vec<u8>> = SchemaColumn::<Vec<u8>> {
    name: "details",
    type_of_col: &TypeOfCol::Blob,
    can_be_null: true,
    _marker: PhantomData,
};

pub const DETAILS_TYPE: SchemaColumn<String> = SchemaColumn::<String> {
    name: "details_type",
    type_of_col: &TypeOfCol::Text,
    can_be_null: true,
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

// ---
/*
   new row helper here
*/
// ---
use my_yrs_lib::YrsError;
use protocol::payload::ColumnValue;
use protocol::row_col::Col;

#[uniffi::export]
pub fn new_log_row(
    timestamp: i64,               // unix seconds
    level: String,                // "INFO" | "WARN" | "ERROR"
    category: String,             // feature area, e.g. "VIEW_PAGE"
    source: String,               // finer-grained origin: file/class/function
    session_id: String,           // app session id
    message: String,              // log text
    details: Option<Vec<u8>>,     // error message, or serialized rust struct
    details_type: Option<String>, // how to deserialize details, if it's a struct
) -> Result<Vec<ColumnValue>, YrsError> {
    Ok(vec![
        ColumnValue {
            column_name: TIMESTAMP.name.to_string(),
            value: Col::Integer(timestamp),
        },
        ColumnValue {
            column_name: LEVEL.name.to_string(),
            value: Col::Text(level),
        },
        ColumnValue {
            column_name: CATEGORY.name.to_string(),
            value: Col::Text(category),
        },
        ColumnValue {
            column_name: SOURCE.name.to_string(),
            value: Col::Text(source),
        },
        ColumnValue {
            column_name: SESSION_ID.name.to_string(),
            value: Col::Text(session_id),
        },
        ColumnValue {
            column_name: MESSAGE.name.to_string(),
            value: Col::Text(message),
        },
        ColumnValue {
            column_name: DETAILS.name.to_string(),
            value: details
                .map(Col::Blob)
                .unwrap_or(Col::Null(StructRepresentingNull {})),
        },
        ColumnValue {
            column_name: DETAILS_TYPE.name.to_string(),
            value: details_type
                .map(Col::Text)
                .unwrap_or(Col::Null(StructRepresentingNull {})),
        },
    ])
}
