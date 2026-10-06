use std::{collections::HashMap, sync::Arc};

use client_table_blueprints::tbl_every_block_in_existence::{self, *};
use client_table_blueprints::tbl_pages::{self, BLOBBED_PAGE, PAGE_ID};
use data_builder_for_operations_that_need_to_be_correct::page::edit_block;
use error_stuff::CqrsErr;
use executor_of_what_the_builder_built_because_the_builder_shouldnt_touch_the_db::page_executor::edit_block_requires_three_db_inserts;
use my_yrs_lib::{BossOfYrs, YrsError, doc_from_snapshot, yrs_error::ErrorInfo};
use protocol::{
    error::DbError,
    payload::*,
    schema_helper::{DestructDbReturnCol, SchemaColumn},
};
use text_diff::diff_logic::get_diff;
use web_internal_db::db_helper;

pub async fn get_title_and_id_of_all_menu_pages() -> Result<GetDataOut, DbError> {
    let arguments = SelectArguments::Three {
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
    };
    let get_data_in = GetDataIn {
        table_name: tbl_every_block_in_existence::TABLE_NAME.into(),
        arguments,
        columns_to_read: vec![
            CONTENT.name.to_string(),
            MY_ID_AS_GIVEN_BY_YRS.name.to_string(),
        ],
    };
    db_helper::get_data(get_data_in).await
}

pub fn destruct_get_title_and_id_of_all_menu_pages(
    row: protocol::row_col::Row,
) -> (String, String) {
    let schema_by_position: HashMap<u8, &SchemaColumn<String>> =
        HashMap::from([(0, &CONTENT), (1, &MY_ID_AS_GIVEN_BY_YRS)]);

    let mut destructed_by_name: HashMap<&str, String> = HashMap::new();

    let mut ctr: u8 = 0;
    let mut iter = row.cols.into_iter();
    while let Some(col) = iter.next() {
        let schema = schema_by_position.get(&ctr).unwrap();
        let value = schema.destruct_db_col(col).unwrap();
        destructed_by_name.insert(schema.name, value);
        ctr += 1;
    }

    let title = destructed_by_name.remove(CONTENT.name).unwrap();
    let yrs_id = destructed_by_name
        .remove(MY_ID_AS_GIVEN_BY_YRS.name)
        .unwrap();

    (title, yrs_id)
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
