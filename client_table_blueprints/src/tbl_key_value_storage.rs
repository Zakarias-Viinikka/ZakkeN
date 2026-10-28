use std::marker::PhantomData;

use my_yrs_lib::YrsError;
use protocol::new_table::ColumnDef;
use protocol::new_table::ColumnType;
use protocol::new_table::id_column;
use protocol::new_table::not_null_col;
use protocol::new_table::not_null_unique_col;
use protocol::payload::ColumnValue;
use protocol::schema_helper::{SchemaColumn, TypeOfCol};

use crate::row_values::text_value;

pub const TABLE_NAME: &str = "key_value_storage";

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

#[uniffi::export]
pub fn new_table_key_value_storage() -> Vec<ColumnDef> {
    vec![
        id_column(),
        not_null_unique_col(ColumnType::Text, KEY.name),
        not_null_col(ColumnType::Text, VALUE.name),
    ]
}

#[uniffi::export]
pub fn new_key_value_item(key: String, value: String) -> Result<Vec<ColumnValue>, YrsError> {
    Ok(vec![text_value(&KEY, key), text_value(&VALUE, value)])
}