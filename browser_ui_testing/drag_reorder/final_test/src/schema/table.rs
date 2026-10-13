#![allow(non_snake_case)]
use std::marker::PhantomData;
use protocol::new_table::{ColumnDef, ColumnType, id_column, not_null_unique_col, not_null_col, ForeignKeyDef};
use protocol::schema_helper::{SchemaColumn, TypeOfCol};
use crate::schema::migration::helpers::ADbTable;

pub const PAGES_TABLE_NAME: &str = "pages";
pub const BLOCKS_TABLE_NAME: &str = "every_block";

pub fn new_table_pages() -> Vec<ColumnDef> {
    vec![
        id_column(),
        not_null_unique_col(ColumnType::Text, "page_id"),
    ]
}

pub fn new_table_every_block() -> Vec<ColumnDef> {
    vec![
        id_column(),
        not_null_unique_col(ColumnType::Text, "block_id"),
        not_null_col(ColumnType::Text, "page_id"),
        not_null_col(ColumnType::Text, "content"),
    ]
}

pub fn get_foreign_def_every_block() -> Vec<ForeignKeyDef> {
    vec![ForeignKeyDef {
        column_name: "page_id".to_string(),
        referenced_table_name: PAGES_TABLE_NAME.to_string(),
        referenced_column_name: "page_id".to_string(),
    }]
}

pub fn all_tables() -> Vec<ADbTable> {
    vec![
        ADbTable {
            table_name: PAGES_TABLE_NAME.to_string(),
            columns: new_table_pages(),
            foreign_keys: Vec::new(),
        },
        ADbTable {
            table_name: BLOCKS_TABLE_NAME.to_string(),
            columns: new_table_every_block(),
            foreign_keys: get_foreign_def_every_block(),
        },
    ]
}
