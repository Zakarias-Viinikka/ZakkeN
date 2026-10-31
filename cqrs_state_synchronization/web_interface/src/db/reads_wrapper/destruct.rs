use client_table_blueprints::tbl_every_block_in_existence::{
    CONTENT, FRAC_POSITION, ID_OF_PAGE_I_BELONG_TO, IS_TITLE, MY_ID_AS_GIVEN_BY_YRS,
};
use protocol::{
    error::DbError,
    row_col::{Col, Row},
    schema_helper::DestructDbReturnCol,
};

use crate::shared_owners::LocalPages;

pub fn destruct_entire_get_all_menu_pages(row: Row) -> Result<LocalPages, DbError> {
    let mut iter = row.cols.into_iter();
    let id_col = iter
        .next()
        .ok_or_else(|| DbError::ColDestructFail("row missing id".into()))?;
    let title_col = iter
        .next()
        .ok_or_else(|| DbError::ColDestructFail("row missing content".into()))?;
    let block_id_col = iter
        .next()
        .ok_or_else(|| DbError::ColDestructFail("row missing my_id_as_given_by_yrs".into()))?;
    let page_id_col = iter
        .next()
        .ok_or_else(|| DbError::ColDestructFail("row missing id_of_page_i_belong_to".into()))?;

    Ok(LocalPages {
        db_id: destruct_id(id_col)? as usize,
        title: destruct_title(title_col)?,
        block_id: destruct_block_id(block_id_col)?,
        page_id: destruct_page_id(page_id_col)?,
        is_disabled: false,
    })
}

pub fn destruct_id(col: Col) -> Result<i64, DbError> {
    match col {
        Col::Integer(i) => Ok(i),
        _ => Err(DbError::ColDestructFail("id was not an integer".into())),
    }
}

pub fn destruct_title(col: Col) -> Result<String, DbError> {
    CONTENT
        .try_destruct_db_col(col)
        .map_err(DbError::ColDestructFail)?
        .ok_or_else(|| DbError::ColDestructFail("content was null".into()))
}

pub fn destruct_block_id(col: Col) -> Result<String, DbError> {
    MY_ID_AS_GIVEN_BY_YRS
        .try_destruct_db_col(col)
        .map_err(DbError::ColDestructFail)?
        .ok_or_else(|| DbError::ColDestructFail("my_id_as_given_by_yrs was null".into()))
}

pub fn destruct_page_id(col: Col) -> Result<String, DbError> {
    ID_OF_PAGE_I_BELONG_TO
        .try_destruct_db_col(col)
        .map_err(DbError::ColDestructFail)?
        .ok_or_else(|| DbError::ColDestructFail("id_of_page_i_belong_to was null".into()))
}

pub fn destruct_frac_position(col: Col) -> Result<String, DbError> {
    FRAC_POSITION
        .try_destruct_db_col(col)
        .map_err(DbError::ColDestructFail)?
        .ok_or_else(|| DbError::ColDestructFail("frac_position was null".into()))
}

pub fn destruct_is_title(col: Col) -> Result<bool, DbError> {
    let s = IS_TITLE
        .try_destruct_db_col(col)
        .map_err(DbError::ColDestructFail)?
        .ok_or_else(|| DbError::ColDestructFail("is_title was null".into()))?;
    Ok(s == "true")
}