use std::sync::Arc;

use client_table_blueprints::tbl_every_block_in_existence::{self, *};
use client_table_blueprints::tbl_pages::{self, BLOBBED_PAGE, PAGE_ID};
use data_builder_for_operations_that_need_to_be_correct::edit_block;
use error_stuff::CqrsErr;
use executor_of_what_the_builder_built_because_the_builder_shouldnt_touch_the_db::page_executor::edit_block_requires_three_db_inserts;
use my_yrs_lib::{BossOfYrs, YrsError, doc_from_snapshot, yrs_error::ErrorInfo};
use protocol::{error::DbError, payload::*, schema_helper::DestructDbReturnCol};
use text_diff::diff_logic::get_diff;
use web_internal_db::db_helper;

use crate::complicated_signal_flow::{BlockRow, LocalPages};
use crate::db::db_out_destruct_helpers::destruct_entire_get_all_menu_pages;

pub async fn get_all_menu_pages() -> Result<Vec<LocalPages>, DbError> {
    let get_data_in = GetDataIn {
        table_name: tbl_every_block_in_existence::TABLE_NAME.into(),
        arguments: SelectArguments::Three {
            first: SelectArgument::XEqualY {
                x: IS_TITLE.name.to_string(),
                y: "true".to_string(),
            },
            join: JoinType::And,
            second: SelectArgument::XEqualY {
                x: IS_PART_OF_MAIN_MENU_PAGE.name.to_string(),
                y: "true".to_string(),
            },
            join2: JoinType::And,
            third: SelectArgument::XEqualY {
                x: PAGE_IS_DISABLED.name.to_string(),
                y: "false".to_string(),
            },
        },
        columns_to_read: vec![
            "id".to_string(),
            CONTENT.name.to_string(),
            MY_ID_AS_GIVEN_BY_YRS.name.to_string(),
            ID_OF_PAGE_I_BELONG_TO.name.to_string(),
        ],
    };
    let out = db_helper::get_data(get_data_in).await?;

    let mut result = Vec::new();
    for row in out.rows {
        result.push(destruct_entire_get_all_menu_pages(row)?);
    }
    Ok(result)
}

pub async fn edit_title_for_page(
    yrs: Arc<BossOfYrs>,
    block_id: String,
    new_title: String,
    session_id: String,
) -> Result<(), CqrsErr> {
    let old_text = yrs.clone().read_block(block_id.clone())?.ok_or_else(|| {
        CqrsErr::YrsErrorContainer(YrsError::GenericError {
            info: ErrorInfo {
                error_msg: format!("no block with id: {block_id}"),
                file: file!().to_string(),
                method: "edit_title_for_page".to_string(),
            },
        })
    })?;

    let diff = get_diff(&old_text, &new_title);

    let ctx = match edit_block(yrs, block_id, diff, session_id)? {
        Some(ctx) => ctx,
        None => return Ok(()),
    };

    edit_block_requires_three_db_inserts(ctx).await?;

    Ok(())
}

pub async fn get_yrs_unblobbed(
    page_id: String,
    user_id: String,
    time: String,
) -> Result<Arc<BossOfYrs>, DbError> {
    let arguments = SelectArguments::Single(SelectArgument::XEqualY {
        x: PAGE_ID.name.to_string(),
        y: page_id.clone(),
    });

    let out = db_helper::get_single_col(GetSingleColIn {
        table_name: tbl_pages::TABLE_NAME.into(),
        arguments,
        column_to_read: BLOBBED_PAGE.name.to_string(),
    })
    .await?;

    let blob = BLOBBED_PAGE
        .destruct_db_col(out.value)
        .ok_or_else(|| DbError::ColDestructFail("blobbed_page was null".to_string()))?;

    let boss = doc_from_snapshot(blob, user_id, page_id, time)
        .map_err(|e| DbError::IllegalInput(format!("doc_from_snapshot failed: {e:?}")))?;

    Ok(Arc::new(boss))
}

pub async fn get_all_blocks_for_page(page_id: String) -> Result<Vec<BlockRow>, DbError> {
    let out = db_helper::get_data(GetDataIn {
        table_name: tbl_every_block_in_existence::TABLE_NAME.into(),
        arguments: SelectArguments::Single(SelectArgument::XEqualY {
            x: tbl_every_block_in_existence::ID_OF_PAGE_I_BELONG_TO
                .name
                .to_string(),
            y: page_id,
        }),
        columns_to_read: vec![
            tbl_every_block_in_existence::MY_ID_AS_GIVEN_BY_YRS
                .name
                .to_string(),
            tbl_every_block_in_existence::ID_OF_PAGE_I_BELONG_TO
                .name
                .to_string(),
            tbl_every_block_in_existence::CONTENT.name.to_string(),
            tbl_every_block_in_existence::IS_TITLE.name.to_string(),
        ],
    })
    .await?;

    let mut result = Vec::new();
    for row in out.rows {
        let mut cols = row.cols.into_iter();
        let block_id = MY_ID_AS_GIVEN_BY_YRS
            .try_destruct_db_col(
                cols.next()
                    .ok_or_else(|| DbError::ColDestructFail("missing block_id".into()))?,
            )
            .map_err(DbError::ColDestructFail)?
            .ok_or_else(|| DbError::ColDestructFail("block_id was null".into()))?;
        let page_id = ID_OF_PAGE_I_BELONG_TO
            .try_destruct_db_col(
                cols.next()
                    .ok_or_else(|| DbError::ColDestructFail("missing page_id".into()))?,
            )
            .map_err(DbError::ColDestructFail)?
            .ok_or_else(|| DbError::ColDestructFail("page_id was null".into()))?;
        let content = CONTENT
            .try_destruct_db_col(
                cols.next()
                    .ok_or_else(|| DbError::ColDestructFail("missing content".into()))?,
            )
            .map_err(DbError::ColDestructFail)?
            .ok_or_else(|| DbError::ColDestructFail("content was null".into()))?;
        let is_title_str = IS_TITLE
            .try_destruct_db_col(
                cols.next()
                    .ok_or_else(|| DbError::ColDestructFail("missing is_title".into()))?,
            )
            .map_err(DbError::ColDestructFail)?
            .ok_or_else(|| DbError::ColDestructFail("is_title was null".into()))?;
        result.push(BlockRow {
            block_id,
            page_id,
            content,
            is_title: is_title_str == "true",
        });
    }
    Ok(result)
}
