#![allow(non_snake_case)]
use std::marker::PhantomData;

use protocol::new_table::{ColumnDef, ColumnType, default_col, id_column, not_null_col};
use protocol::schema_helper::{SchemaColumn, TypeOfCol};

pub fn new_table_drift_version() -> Vec<ColumnDef> {
    vec![
        id_column(),
        not_null_col(ColumnType::Text, "version"),
        not_null_col(ColumnType::Text, "reason"),
        default_col(ColumnType::Text, "drift_solution"),
    ]
}

pub const TABLE_NAME: &str = "drift_version";

pub const VERSION: SchemaColumn<String> = SchemaColumn::<String> {
    name: "version",
    type_of_col: &TypeOfCol::Text,
    can_be_null: false,
    _marker: PhantomData,
};

pub const REASON: SchemaColumn<String> = SchemaColumn::<String> {
    name: "reason",
    type_of_col: &TypeOfCol::Text,
    can_be_null: false,
    _marker: PhantomData,
};

/// How stale clients should react to this drift version: wipe and
/// adopt, reconcile, wipe and pull from server, etc. Empty when the
/// bump was automatic with no instruction attached, in which case the
/// client shows the user a picker.
pub const DRIFT_SOLUTION: SchemaColumn<String> = SchemaColumn::<String> {
    name: "drift_solution",
    type_of_col: &TypeOfCol::Text,
    can_be_null: true,
    _marker: PhantomData,
};
