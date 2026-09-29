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

pub enum ColumnsKeyValueStorage {
    Key,
    Value,
}

pub fn get_table_name_key_value_storage() -> String {
    "key_value_storage".into()
}

const KEY: SchemaColumn<String> = SchemaColumn::<String> {
    name: "key",
    type_of_col: &TypeOfCol::Text,
    _marker: PhantomData,
};

const VALUE: SchemaColumn<String> = SchemaColumn::<String> {
    name: "value",
    type_of_col: &TypeOfCol::Text,
    _marker: PhantomData,
};

pub fn get_column_name(ENUM: ColumnsKeyValueStorage) -> String {
    match ENUM {
        ColumnsKeyValueStorage::Key => KEY.name.to_string(),
        ColumnsKeyValueStorage::Value => VALUE.name.to_string(),
    }
}

pub fn destruct_col_key_value_storage_key(
    col_to_destruct: protocol::row_col::Col,
) -> Result<String, DbError> {
    KEY.destruct_db_col(col_to_destruct).map_err(|_| {
        ColDestructFail(format!(
            "Failed  to destruct to type: {:?}",
            KEY.type_of_col
        ))
    })
}

pub fn destruct_col_key_value_storage_value(
    col_to_destruct: protocol::row_col::Col,
) -> Result<String, DbError> {
    VALUE.destruct_db_col(col_to_destruct).map_err(|_| {
        ColDestructFail(format!(
            "Failed  to destruct to type: {:?}",
            VALUE.type_of_col
        ))
    })
}
