use std::marker::PhantomData;

use protocol::new_table::ColumnDef;
use protocol::new_table::ColumnType;
use protocol::new_table::id_column;
use protocol::new_table::not_null_col;
use protocol::payload::ColumnValue;
use protocol::row_col::Col;
use protocol::schema_helper::{SchemaColumn, TypeOfCol};

pub const TABLE_NAME: &str = "seen_server_letters";

pub const LETTER_ID: SchemaColumn<i64> = SchemaColumn::<i64> {
    name: "letter_id",
    type_of_col: &TypeOfCol::Integer,
    can_be_null: false,
    _marker: PhantomData,
};

#[uniffi::export]
pub fn new_table_seen_server_letters() -> Vec<ColumnDef> {
    vec![
        id_column(),
        not_null_col(ColumnType::Integer, LETTER_ID.name),
    ]
}

#[uniffi::export]
pub fn new_seen_server_letter_row(letter_id: i64) -> Vec<ColumnValue> {
    vec![ColumnValue {
        column_name: LETTER_ID.name.to_string(),
        value: Col::Integer(letter_id),
    }]
}