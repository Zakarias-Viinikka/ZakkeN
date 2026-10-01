#![allow(non_snake_case)]
use std::marker::PhantomData;

//use protocol::error::DbError;
//use protocol::error::DbError::ColDestructFail;
//use protocol::schema_helper::DestructDbReturnCol;
use protocol::new_table::ColumnDef;
use protocol::new_table::ColumnType;
use protocol::new_table::id_column;
use protocol::new_table::not_null_col;
use protocol::new_table::not_null_unique_col;
use protocol::schema_helper::{SchemaColumn, TypeOfCol};

#[uniffi::export]
pub fn new_table_key_value_storage() -> Vec<ColumnDef> {
    vec![
        id_column(),
        not_null_unique_col(ColumnType::Text, "key"),
        not_null_col(ColumnType::Text, "value"),
    ]
}

pub enum ColumnsKeyValueStorage {
    Key,
    Value,
}

pub fn get_table_name_key_value_storage() -> String {
    "key_value_storage".into()
}

pub const KEY: SchemaColumn<String> = SchemaColumn::<String> {
    name: "key",
    type_of_col: &TypeOfCol::Text,
    can_be_null: false,
    _marker: PhantomData,
};

pub const VALUE: SchemaColumn<String> = SchemaColumn::<String> {
    name: "value",
    type_of_col: &TypeOfCol::Text,
    can_be_null: false,
    _marker: PhantomData,
};

pub fn get_column_name(ENUM: ColumnsKeyValueStorage) -> String {
    match ENUM {
        ColumnsKeyValueStorage::Key => KEY.name.to_string(),
        ColumnsKeyValueStorage::Value => VALUE.name.to_string(),
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
pub fn new_key_value_item(key: String, value: String) -> Result<Vec<ColumnValue>, YrsError> {
    Ok(vec![
        ColumnValue {
            column_name: KEY.name.to_string(),
            value: Col::Text(key),
        },
        ColumnValue {
            column_name: VALUE.name.to_string(),
            value: Col::Text(value),
        },
    ])
}
