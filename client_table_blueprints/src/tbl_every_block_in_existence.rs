use std::marker::PhantomData;

use my_yrs_lib::YrsError;
use protocol::new_table::ColumnDef;
use protocol::new_table::ColumnType;
use protocol::new_table::ForeignKeyDef;
use protocol::new_table::id_column;
use protocol::new_table::not_null_col;
use protocol::payload::ColumnValue;
use protocol::schema_helper::{SchemaColumn, TypeOfCol};

use crate::row_values::text_value;

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

pub const FRAC_POSITION: SchemaColumn<String> = SchemaColumn::<String> {
    name: "frac_position",
    type_of_col: &TypeOfCol::Text,
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
pub fn new_table_every_block_in_existence() -> Vec<ColumnDef> {
    vec![
        id_column(),
        not_null_col(ColumnType::Text, IS_TITLE.name),
        not_null_col(ColumnType::Text, IS_PART_OF_MAIN_MENU_PAGE.name),
        not_null_col(ColumnType::Text, CONTENT.name),
        not_null_col(ColumnType::Text, MY_ID_AS_GIVEN_BY_YRS.name),
        not_null_col(ColumnType::Text, ID_OF_PAGE_I_BELONG_TO.name),
        not_null_col(ColumnType::Text, FRAC_POSITION.name),
        not_null_col(ColumnType::Text, PAGE_IS_DISABLED.name),
    ]
}

#[uniffi::export]
pub fn get_foreign_def_every_block_in_existence() -> Vec<ForeignKeyDef> {
    vec![ForeignKeyDef {
        column_name: ID_OF_PAGE_I_BELONG_TO.name.to_string(),
        referenced_table_name: crate::tbl_pages::TABLE_NAME.to_string(),
        referenced_column_name: crate::tbl_pages::PAGE_ID.name.to_string(),
    }]
}

#[uniffi::export]
pub fn new_every_block_in_existence_row(
    is_title: bool,
    is_part_of_main_menu_page: bool,
    content: String,
    my_id_as_given_by_yrs: String,
    id_of_page_i_belong_to: String,
    frac_position: String,
) -> Result<Vec<ColumnValue>, YrsError> {
    Ok(vec![
        text_value(&IS_TITLE, is_title.to_string()),
        text_value(&IS_PART_OF_MAIN_MENU_PAGE, is_part_of_main_menu_page.to_string()),
        text_value(&CONTENT, content),
        text_value(&MY_ID_AS_GIVEN_BY_YRS, my_id_as_given_by_yrs),
        text_value(&ID_OF_PAGE_I_BELONG_TO, id_of_page_i_belong_to),
        text_value(&FRAC_POSITION, frac_position),
        text_value(&PAGE_IS_DISABLED, "false".to_string()),
    ])
}