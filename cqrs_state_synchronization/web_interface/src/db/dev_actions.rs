use client_table_blueprints::{
    tbl_client_love_letter, tbl_every_block_in_existence, tbl_key_value_storage, tbl_pages,
    tbl_seen_server_letters,
};
use data_builder_for_operations_that_need_to_be_correct::{add_block, create_page};
use error_stuff::CqrsErr;
use executor_of_what_the_builder_built_because_the_builder_shouldnt_touch_the_db::page_executor::{
    add_block_requires_three_db_actions, insert_page_requires_three_db_inserts,
    run_in_transaction,
};
use my_yrs_lib::yrs_wrapper::PositionToInsert;
use protocol::payload::DeleteAllRowsIn;
use web_internal_db::db_helper;

use crate::db::cached_ids::{current_session_id, current_user_id};
use crate::db::reads::pages::get_yrs_unblobbed;
use crate::db::write_lock::WRITE_LOCK;
use crate::db::writes::edit_title_for_page;
use crate::get_time;

pub async fn insert_three_pages() -> Result<(), CqrsErr> {
    let seeds: [(&str, &[&str]); 3] = [
        (
            "Grocery list",
            &["Milk", "Eggs", "Bread", "Butter", "Coffee beans"],
        ),
        (
            "Work notes",
            &[
                "Finish the report by Friday",
                "Call the client back",
                "Review PR #42",
            ],
        ),
        (
            "Ideas",
            &[
                "A notes app that syncs offline",
                "A drag-reorder UI without dependencies",
            ],
        ),
    ];

    run_in_transaction(|| async move {
        for (title, body) in seeds {
            let session_id = current_session_id();

            let everything = create_page(true, session_id.clone())?;
            let title_block_id = everything
                .blocks_to_insert
                .title_block
                .my_id_as_given_by_yrs
                .clone();
            let first_normal_id = everything
                .blocks_to_insert
                .normal_block
                .my_id_as_given_by_yrs
                .clone();
            let page_id = everything.page_to_insert.page_id.clone();
            insert_page_requires_three_db_inserts(everything).await?;

            let boss = get_yrs_unblobbed(page_id, current_user_id(), get_time()).await?;

            edit_title_for_page(
                boss.clone(),
                title_block_id,
                title.to_string(),
                session_id.clone(),
            )
            .await?;

            if let Some(first) = body.first() {
                edit_title_for_page(
                    boss.clone(),
                    first_normal_id,
                    first.to_string(),
                    session_id.clone(),
                )
                .await?;
            }

            for extra in body.iter().skip(1) {
                let ctx = add_block(
                    boss.clone(),
                    extra.to_string(),
                    String::new(),
                    PositionToInsert::AtEnd,
                    true,
                    session_id.clone(),
                    None,
                )?;
                add_block_requires_three_db_actions(ctx).await?;
            }
        }
        Ok(())
    })
    .await
}

pub async fn delete_everything() -> Result<(), CqrsErr> {
    let _guard = WRITE_LOCK.acquire().await;
    wipe_tables(&[
        tbl_pages::TABLE_NAME,
        tbl_every_block_in_existence::TABLE_NAME,
        tbl_client_love_letter::TABLE_NAME,
        tbl_seen_server_letters::TABLE_NAME,
    ])
    .await
}

pub async fn reset_everything() -> Result<(), CqrsErr> {
    let _guard = WRITE_LOCK.acquire().await;
    wipe_tables(&[
        tbl_pages::TABLE_NAME,
        tbl_every_block_in_existence::TABLE_NAME,
        tbl_client_love_letter::TABLE_NAME,
        tbl_seen_server_letters::TABLE_NAME,
        tbl_key_value_storage::TABLE_NAME,
    ])
    .await
}

async fn wipe_tables(tables: &[&str]) -> Result<(), CqrsErr> {
    for table_name in tables {
        db_helper::delete_all_rows(DeleteAllRowsIn {
            table_name: table_name.to_string(),
        })
        .await?;
    }
    Ok(())
}