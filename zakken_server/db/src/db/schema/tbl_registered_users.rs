#![allow(non_snake_case)]
use std::marker::PhantomData;

use protocol::new_table::{ColumnDef, ColumnType, id_column, not_null_unique_col};
use protocol::schema_helper::{SchemaColumn, TypeOfCol};

pub fn new_table_registered_users() -> Vec<ColumnDef> {
    vec![
        id_column(),
        not_null_unique_col(ColumnType::Text, "user_name"),
    ]
}

pub const TABLE_NAME: &str = "registered_users";

pub const USER_NAME: SchemaColumn<String> = SchemaColumn::<String> {
    name: "user_name",
    type_of_col: &TypeOfCol::Text,
    can_be_null: false,
    _marker: PhantomData,
};
