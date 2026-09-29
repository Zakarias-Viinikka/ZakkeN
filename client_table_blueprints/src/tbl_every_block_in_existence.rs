#![allow(non_snake_case)]
use std::marker::PhantomData;

use my_yrs_lib::YrsError;
use protocol::error::DbError;
use protocol::error::DbError::ColDestructFail;
use protocol::new_table::ForeignKeyDef;
use protocol::payload::ColumnValue;
use protocol::row_col::Col;
use protocol::schema_helper::DestructDbReturnCol;
use protocol::schema_helper::{SchemaColumn, TypeOfCol};

pub enum ColumnsEveryBlockInExistence {
    IsTitle,
    IsPartOfMainMenuPage,
    Content,
    MyIdAsGivenByYrs,
    IdOfPageIBelongTo,
    Position,
}

pub fn get_table_name_every_block_in_existence() -> String {
    "every_block_in_existence".into()
}

const IS_TITLE: SchemaColumn<String> = SchemaColumn::<String> {
    name: "is_title",
    type_of_col: &TypeOfCol::Text,
    _marker: PhantomData,
};

const IS_PART_OF_MAIN_MENU_PAGE: SchemaColumn<String> = SchemaColumn::<String> {
    name: "is_part_of_main_menu_page",
    type_of_col: &TypeOfCol::Text,
    _marker: PhantomData,
};

const CONTENT: SchemaColumn<String> = SchemaColumn::<String> {
    name: "content",
    type_of_col: &TypeOfCol::Text,
    _marker: PhantomData,
};

const MY_ID_AS_GIVEN_BY_YRS: SchemaColumn<String> = SchemaColumn::<String> {
    name: "my_id_as_given_by_yrs",
    type_of_col: &TypeOfCol::Text,
    _marker: PhantomData,
};

const ID_OF_PAGE_I_BELONG_TO: SchemaColumn<String> = SchemaColumn::<String> {
    name: "id_of_page_i_belong_to",
    type_of_col: &TypeOfCol::Text,
    _marker: PhantomData,
};

const POSITION: SchemaColumn<f64> = SchemaColumn::<f64> {
    name: "position",
    type_of_col: &TypeOfCol::Real,
    _marker: PhantomData,
};

pub fn get_column_name(ENUM: ColumnsEveryBlockInExistence) -> String {
    match ENUM {
        ColumnsEveryBlockInExistence::IsTitle => IS_TITLE.name.to_string(),
        ColumnsEveryBlockInExistence::IsPartOfMainMenuPage => {
            IS_PART_OF_MAIN_MENU_PAGE.name.to_string()
        }
        ColumnsEveryBlockInExistence::Content => CONTENT.name.to_string(),
        ColumnsEveryBlockInExistence::MyIdAsGivenByYrs => MY_ID_AS_GIVEN_BY_YRS.name.to_string(),
        ColumnsEveryBlockInExistence::IdOfPageIBelongTo => ID_OF_PAGE_I_BELONG_TO.name.to_string(),
        ColumnsEveryBlockInExistence::Position => POSITION.name.to_string(),
    }
}

pub fn destruct_col_every_block_in_existence_is_title(
    col_to_destruct: protocol::row_col::Col,
) -> Result<String, DbError> {
    IS_TITLE.destruct_db_col(col_to_destruct).map_err(|_| {
        ColDestructFail(format!(
            "Failed  to destruct to type: {:?}",
            IS_TITLE.type_of_col
        ))
    })
}

pub fn destruct_col_every_block_in_existence_is_part_of_main_menu_page(
    col_to_destruct: protocol::row_col::Col,
) -> Result<String, DbError> {
    IS_PART_OF_MAIN_MENU_PAGE
        .destruct_db_col(col_to_destruct)
        .map_err(|_| {
            ColDestructFail(format!(
                "Failed  to destruct to type: {:?}",
                IS_PART_OF_MAIN_MENU_PAGE.type_of_col
            ))
        })
}

pub fn destruct_col_every_block_in_existence_content(
    col_to_destruct: protocol::row_col::Col,
) -> Result<String, DbError> {
    CONTENT.destruct_db_col(col_to_destruct).map_err(|_| {
        ColDestructFail(format!(
            "Failed  to destruct to type: {:?}",
            CONTENT.type_of_col
        ))
    })
}

pub fn destruct_col_every_block_in_existence_my_id_as_given_by_yrs(
    col_to_destruct: protocol::row_col::Col,
) -> Result<String, DbError> {
    MY_ID_AS_GIVEN_BY_YRS
        .destruct_db_col(col_to_destruct)
        .map_err(|_| {
            ColDestructFail(format!(
                "Failed  to destruct to type: {:?}",
                MY_ID_AS_GIVEN_BY_YRS.type_of_col
            ))
        })
}

pub fn destruct_col_every_block_in_existence_id_of_page_i_belong_to(
    col_to_destruct: protocol::row_col::Col,
) -> Result<String, DbError> {
    ID_OF_PAGE_I_BELONG_TO
        .destruct_db_col(col_to_destruct)
        .map_err(|_| {
            ColDestructFail(format!(
                "Failed  to destruct to type: {:?}",
                ID_OF_PAGE_I_BELONG_TO.type_of_col
            ))
        })
}

pub fn destruct_col_every_block_in_existence_position(
    col_to_destruct: protocol::row_col::Col,
) -> Result<f64, DbError> {
    POSITION.destruct_db_col(col_to_destruct).map_err(|_| {
        ColDestructFail(format!(
            "Failed  to destruct to type: {:?}",
            POSITION.type_of_col
        ))
    })
}

#[uniffi::export]
pub fn new_every_block_in_existence_row(
    is_title: String,
    is_part_of_main_menu_page: String,
    content: String,
    my_id_as_given_by_yrs: String,
    id_of_page_i_belong_to: String,
    position: f64,
) -> Result<Vec<ColumnValue>, YrsError> {
    Ok(vec![
        ColumnValue {
            column_name: IS_TITLE.name.to_string(),
            value: Col::Text(is_title),
        },
        ColumnValue {
            column_name: IS_PART_OF_MAIN_MENU_PAGE.name.to_string(),
            value: Col::Text(is_part_of_main_menu_page),
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
    ])
}

#[uniffi::export]
pub fn get_foreign_def_every_block_in_existence() -> Vec<ForeignKeyDef> {
    vec![ForeignKeyDef {
        column: ID_OF_PAGE_I_BELONG_TO.name.to_string(),
        referenced_table: crate::tbl_pages::get_table_name_pages(),
        referenced_column: crate::tbl_pages::PAGE_ID.name.to_string(),
    }]
}
