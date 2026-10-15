#![allow(non_snake_case)]
use std::marker::PhantomData;

use protocol::new_table::{ColumnDef, ColumnType, id_column, not_null_col};
use protocol::schema_helper::{SchemaColumn, TypeOfCol};

pub fn new_table_love_letters_from_clients() -> Vec<ColumnDef> {
    vec![
        id_column(),
        not_null_col(ColumnType::Blob, "love_letter"),
        not_null_col(ColumnType::Text, "target_page"),
    ]
}

pub const TABLE_NAME: &str = "love_letters_from_clients";

pub const LOVE_LETTER: SchemaColumn<Vec<u8>> = SchemaColumn::<Vec<u8>> {
    name: "love_letter",
    type_of_col: &TypeOfCol::Blob,
    can_be_null: false,
    _marker: PhantomData,
};

pub const TARGET_PAGE: SchemaColumn<String> = SchemaColumn::<String> {
    name: "target_page",
    type_of_col: &TypeOfCol::Text,
    can_be_null: false,
    _marker: PhantomData,
};
