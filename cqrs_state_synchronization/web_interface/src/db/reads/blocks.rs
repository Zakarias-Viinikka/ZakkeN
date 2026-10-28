use client_table_blueprints::tbl_every_block_in_existence::{
    self, CONTENT, FRAC_POSITION, ID_OF_PAGE_I_BELONG_TO, IS_TITLE, MY_ID_AS_GIVEN_BY_YRS,
};
use data_builder_for_operations_that_need_to_be_correct::ExistingBlock;
use protocol::error::DbError;
use protocol::payload::*;
use protocol::row_col::{Col, Row};
use web_internal_db::db_helper;

use crate::complicated_signal_flow::BlockRow;
use super::destruct::{
    destruct_block_id, destruct_frac_position, destruct_is_title, destruct_page_id, destruct_title,
};

async fn blocks_in_frac_order(
    page_id: String,
    columns_to_read: Vec<String>,
) -> Result<Vec<Row>, DbError> {
    let order_by = format!(
        "{} ASC, {} ASC",
        FRAC_POSITION.name, MY_ID_AS_GIVEN_BY_YRS.name,
    );
    let out = db_helper::get_data_ordered(GetDataOrderedIn {
        table_name: tbl_every_block_in_existence::TABLE_NAME.into(),
        arguments: SelectArguments::Single(SelectArgument::XEqualY {
            x: ID_OF_PAGE_I_BELONG_TO.name.to_string(),
            y: page_id,
        }),
        columns_to_read,
        order_by,
    })
    .await?;
    Ok(out.rows)
}

fn next_col(cols: &mut std::vec::IntoIter<Col>, name: &str) -> Result<Col, DbError> {
    cols.next()
        .ok_or_else(|| DbError::ColDestructFail(format!("missing {name}")))
}

pub async fn get_all_blocks_for_page(page_id: String) -> Result<Vec<BlockRow>, DbError> {
    let rows = blocks_in_frac_order(
        page_id,
        vec![
            MY_ID_AS_GIVEN_BY_YRS.name.to_string(),
            ID_OF_PAGE_I_BELONG_TO.name.to_string(),
            CONTENT.name.to_string(),
            IS_TITLE.name.to_string(),
        ],
    )
    .await?;
    let mut result = Vec::with_capacity(rows.len());
    for row in rows {
        let mut cols = row.cols.into_iter();
        result.push(BlockRow {
            block_id: destruct_block_id(next_col(&mut cols, "block_id")?)?,
            page_id: destruct_page_id(next_col(&mut cols, "page_id")?)?,
            content: destruct_title(next_col(&mut cols, "content")?)?,
            is_title: destruct_is_title(next_col(&mut cols, "is_title")?)?,
        });
    }
    Ok(result)
}

pub async fn get_existing_blocks_for_page(page_id: String) -> Result<Vec<ExistingBlock>, DbError> {
    let rows = blocks_in_frac_order(
        page_id,
        vec![
            MY_ID_AS_GIVEN_BY_YRS.name.to_string(),
            CONTENT.name.to_string(),
            FRAC_POSITION.name.to_string(),
            IS_TITLE.name.to_string(),
        ],
    )
    .await?;
    let mut result = Vec::with_capacity(rows.len());
    for row in rows {
        let mut cols = row.cols.into_iter();
        result.push(ExistingBlock {
            block_id: destruct_block_id(next_col(&mut cols, "block_id")?)?,
            content: destruct_title(next_col(&mut cols, "content")?)?,
            frac_position: destruct_frac_position(next_col(&mut cols, "frac_position")?)?,
            is_title: destruct_is_title(next_col(&mut cols, "is_title")?)?,
        });
    }
    Ok(result)
}