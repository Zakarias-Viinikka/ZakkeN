#![allow(non_snake_case)]
use std::marker::PhantomData;

use protocol::new_table::{ColumnDef, ColumnType, id_column, not_null_col};
use protocol::schema_helper::{SchemaColumn, TypeOfCol};

pub fn new_table_unsent_love_letters() -> Vec<ColumnDef> {
    vec![
        id_column(),
        not_null_col(ColumnType::Text, "user_name"),
        not_null_col(ColumnType::Integer, "letter_id"),
    ]
}

pub const TABLE_NAME: &str = "unsent_love_letters";

pub const USER_NAME: SchemaColumn<String> = SchemaColumn::<String> {
    name: "user_name",
    type_of_col: &TypeOfCol::Text,
    can_be_null: false,
    _marker: PhantomData,
};

pub const LETTER_ID: SchemaColumn<i64> = SchemaColumn::<i64> {
    name: "letter_id",
    type_of_col: &TypeOfCol::Integer,
    can_be_null: false,
    _marker: PhantomData,
};
