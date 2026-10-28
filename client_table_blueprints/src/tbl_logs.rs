use std::marker::PhantomData;

use my_yrs_lib::YrsError;
use protocol::new_table::ColumnDef;
use protocol::new_table::ColumnType;
use protocol::new_table::id_column;
use protocol::new_table::not_null_col;
use protocol::payload::ColumnValue;
use protocol::row_col::Col;
use protocol::schema_helper::{SchemaColumn, TypeOfCol};

use crate::row_values::text_value;

pub const TABLE_NAME: &str = "logs";

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

#[uniffi::export]
pub fn new_table_logs() -> Vec<ColumnDef> {
    vec![
        id_column(),
        not_null_col(ColumnType::Integer, TIMESTAMP.name),
        not_null_col(ColumnType::Text, LEVEL.name),
        not_null_col(ColumnType::Text, CATEGORY.name),
        not_null_col(ColumnType::Text, SOURCE.name),
        not_null_col(ColumnType::Text, SESSION_ID.name),
        not_null_col(ColumnType::Text, MESSAGE.name),
    ]
}

#[uniffi::export]
pub fn new_log_row(
    timestamp: i64,
    level: String,
    category: String,
    source: String,
    session_id: String,
    message: String,
) -> Result<Vec<ColumnValue>, YrsError> {
    Ok(vec![
        ColumnValue {
            column_name: TIMESTAMP.name.to_string(),
            value: Col::Integer(timestamp),
        },
        text_value(&LEVEL, level),
        text_value(&CATEGORY, category),
        text_value(&SOURCE, source),
        text_value(&SESSION_ID, session_id),
        text_value(&MESSAGE, message),
    ])
}