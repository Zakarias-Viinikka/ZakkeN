#![allow(non_snake_case)]
use std::marker::PhantomData;

use my_yrs_lib::YrsError;
use protocol::new_table::ColumnDef;
use protocol::new_table::ColumnType;
//use protocol::error::DbError;
//use protocol::error::DbError::ColDestructFail;
use protocol::new_table::ForeignKeyDef;
use protocol::new_table::id_column;
use protocol::new_table::not_null_col;
use protocol::payload::ColumnValue;
use protocol::row_col::Col;
//use protocol::schema_helper::DestructDbReturnCol;
use protocol::schema_helper::{SchemaColumn, TypeOfCol};

#[uniffi::export]
pub fn new_table_every_block_in_existence() -> Vec<ColumnDef> {
    vec![
        id_column(),
        not_null_col(ColumnType::Text, "is_title"),
        not_null_col(ColumnType::Text, "is_part_of_main_menu_page"),
        not_null_col(ColumnType::Text, "content"),
        not_null_col(ColumnType::Text, "my_id_as_given_by_yrs"),
        not_null_col(ColumnType::Text, "id_of_page_i_belong_to"),
        not_null_col(ColumnType::Real, "position"),
        not_null_col(ColumnType::Text, "page_is_disabled"),
    ]
}

pub const TABLE_NAME: &str = "every_block_in_existence";

pub const IS_TITLE: SchemaColumn<String> = SchemaColumn::<String> {
    name: "is_title",
    type_of_col: &TypeOfCol::Text,
    can_be_null: false,
    _marker: PhantomData,
};

pub const IS_PART_OF_MAIN_MENU_PAGE: SchemaColumn<String> = SchemaColumn::<String> {
    name: "is_part_of_main_menu_page",
    type_of_col: &TypeOfCol::Text,
    can_be_null: false,
    _marker: PhantomData,
};

pub const CONTENT: SchemaColumn<String> = SchemaColumn::<String> {
    name: "content",
    type_of_col: &TypeOfCol::Text,
    can_be_null: false,
    _marker: PhantomData,
};

pub const MY_ID_AS_GIVEN_BY_YRS: SchemaColumn<String> = SchemaColumn::<String> {
    name: "my_id_as_given_by_yrs",
    type_of_col: &TypeOfCol::Text,
    can_be_null: false,
    _marker: PhantomData,
};

pub const ID_OF_PAGE_I_BELONG_TO: SchemaColumn<String> = SchemaColumn::<String> {
    name: "id_of_page_i_belong_to",
    type_of_col: &TypeOfCol::Text,
    can_be_null: false,
    _marker: PhantomData,
};

pub const POSITION: SchemaColumn<f64> = SchemaColumn::<f64> {
    name: "position",
    type_of_col: &TypeOfCol::Real,
    can_be_null: false,
    _marker: PhantomData,
};

pub const PAGE_IS_DISABLED: SchemaColumn<String> = SchemaColumn::<String> {
    name: "page_is_disabled",
    type_of_col: &TypeOfCol::Text,
    can_be_null: false,
    _marker: PhantomData,
};

#[uniffi::export]
pub fn get_foreign_def_every_block_in_existence() -> Vec<ForeignKeyDef> {
    vec![ForeignKeyDef {
        column_name: ID_OF_PAGE_I_BELONG_TO.name.to_string(),
        referenced_table_name: crate::tbl_pages::TABLE_NAME.to_string(),
        referenced_column_name: crate::tbl_pages::PAGE_ID.name.to_string(),
    }]
}

// ---
/*
   new row helper here
*/
// ---

#[uniffi::export]
pub fn new_every_block_in_existence_row(
    is_title: bool,
    is_part_of_main_menu_page: bool,
    content: String,
    my_id_as_given_by_yrs: String,
    id_of_page_i_belong_to: String,
    position: f64,
) -> Result<Vec<ColumnValue>, YrsError> {
    Ok(vec![
        ColumnValue {
            column_name: IS_TITLE.name.to_string(),
            value: Col::Text(is_title.to_string()),
        },
        ColumnValue {
            column_name: IS_PART_OF_MAIN_MENU_PAGE.name.to_string(),
            value: Col::Text(is_part_of_main_menu_page.to_string()),
        },
        ColumnValue {
            column_name: CONTENT.name.to_string(),
            value: Col::Text(content),
        },
        ColumnValue {
            column_name: MY_ID_AS_GIVEN_BY_YRS.name.to_string(),
            value: Col::Text(my_id_as_given_by_yrs),
        },
        ColumnValue {
            column_name: ID_OF_PAGE_I_BELONG_TO.name.to_string(),
            value: Col::Text(id_of_page_i_belong_to),
        },
        ColumnValue {
            column_name: POSITION.name.to_string(),
            value: Col::Real(position),
        },
        ColumnValue {
            column_name: PAGE_IS_DISABLED.name.to_string(),
            value: Col::Text("false".to_string()),
        },
    ])
}
