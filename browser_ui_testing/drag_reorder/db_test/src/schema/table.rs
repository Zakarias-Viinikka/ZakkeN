#![allow(non_snake_case)]
use std::marker::PhantomData;
use protocol::new_table::{ColumnDef, ColumnType, id_column, not_null_unique_col};
use protocol::payload::ColumnValue;
use protocol::row_col::Col;
use protocol::schema_helper::{SchemaColumn, TypeOfCol};

pub const TABLE_NAME: &str = "words";

pub fn new_table_words() -> Vec<ColumnDef> {
    vec![
        id_column(),
        not_null_unique_col(ColumnType::Text, WORD.name),
    ]
}

pub const WORD: SchemaColumn<String> = SchemaColumn::<String> {
    name: "word",
    type_of_col: &TypeOfCol::Text,
    can_be_null: false,
    _marker: PhantomData,
};

pub fn new_word_row(word: String) -> Vec<ColumnValue> {
    vec![ColumnValue {
        column_name: WORD.name.to_string(),
        value: Col::Text(word),
    }]
}

pub fn all_tables() -> Vec<crate::schema::migration::helpers::ADbTable> {
    vec![crate::schema::migration::helpers::ADbTable {
        table_name: TABLE_NAME.to_string(),
        columns: new_table_words(),
        foreign_keys: Vec::new(),
    }]
}
