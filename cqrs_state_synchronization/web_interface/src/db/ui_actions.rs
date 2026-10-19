use client_table_blueprints::{tbl_client_love_letter, tbl_every_block_in_existence, tbl_pages};
use data_builder_for_operations_that_need_to_be_correct::{
    create_page, disable_page as build_disable_page,
};
use error_stuff::CqrsErr;
use executor_of_what_the_builder_built_because_the_builder_shouldnt_touch_the_db::page_executor::{
    disabled_page_requires_three_db_actions, insert_page_requires_three_db_inserts,
};
use protocol::payload::*;
use web_internal_db::db_helper;

use crate::db::db_helpers_for_web_client;
use crate::db::ui_actions_helper::{get_active_pages_blob, get_page_is_disabled};

async fn write_title_to_block(
    page_id: String,
    block_id: String,
    title: String,
    session_id: String,
) -> Result<(), CqrsErr> {
    let yrs = db_helpers_for_web_client::get_yrs_unblobbed(
        page_id,
        crate::FAKE_USER_ID.to_string(),
        crate::FAKE_TIME.to_string(),
    )
    .await?;

    db_helpers_for_web_client::edit_title_for_page(yrs, block_id, title, session_id).await?;

    Ok(())
}

pub async fn create_new_page() -> Result<(), CqrsErr> {
    let session_id = crate::FAKE_SESSION_ID.to_string();
    let everything = create_page(true, session_id.clone())?;

    let title_block_id = everything
        .blocks_to_insert
        .title_block
        .my_id_as_given_by_yrs
        .clone();
    let page_id = everything.page_to_insert.page_id.clone();

    insert_page_requires_three_db_inserts(everything).await?;

    write_title_to_block(page_id, title_block_id, "title".to_string(), session_id).await?;

    Ok(())
}

pub async fn disable_page(page_id: String, session_id: String) -> Result<(), CqrsErr> {
    if get_page_is_disabled(page_id.clone()).await? {
        return Ok(());
    }

    let active_pages_blob = get_active_pages_blob(page_id.clone()).await?;

    let everything = build_disable_page(page_id, session_id, active_pages_blob)?;

    disabled_page_requires_three_db_actions(everything).await?;

    Ok(())
}

pub async fn delete_everything() -> Result<(), CqrsErr> {
    let tables_to_delete = vec![
        tbl_pages::TABLE_NAME,
        tbl_every_block_in_existence::TABLE_NAME,
        tbl_client_love_letter::TABLE_NAME,
    ];

    for table_name in &tables_to_delete {
        db_helper::delete_all_rows(DeleteAllRowsIn {
            table_name: table_name.to_string(),
        })
        .await?;
    }

    Ok(())
}
