

use data_builder_for_operations_that_need_to_be_correct::{add_block, create_page};
use leptos::prelude::GetUntracked;
use error_stuff::CqrsErr;
use executor_of_what_the_builder_built_because_the_builder_shouldnt_touch_the_db::{
    end_transaction_if_i_own_the_transaction, start_transaction_if_not_in_one,
};
use executor_of_what_the_builder_built_because_the_builder_shouldnt_touch_the_db::page_executor::{
    add_block_requires_three_db_actions, insert_page_requires_three_db_inserts,
};
use my_yrs_lib::yrs_wrapper::PositionToInsert;
use web_internal_db::db_helper;

use crate::db::db_helpers_for_web_client;

pub async fn insert_three_pages() -> Result<(), CqrsErr> {
    let do_i_own_the_transaction = start_transaction_if_not_in_one!();

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

    for (title, body) in seeds {
        let result: Result<(), CqrsErr> = async {
            let session_id = crate::db::cached_ids::SESSION_ID.get_untracked().expect(crate::db::cached_ids::INIT_ERR);

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

            let boss = db_helpers_for_web_client::get_yrs_unblobbed(
                page_id,
                crate::db::cached_ids::USER_ID.get_untracked().expect(crate::db::cached_ids::INIT_ERR),
                crate::get_time(),
            )
            .await?;

            db_helpers_for_web_client::edit_title_for_page(
                boss.clone(),
                title_block_id,
                title.to_string(),
                session_id.clone(),
            )
            .await?;

            if let Some(first) = body.first() {
                db_helpers_for_web_client::edit_title_for_page(
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

            Ok(())
        }
        .await;

        if let Err(e) = result {
            if do_i_own_the_transaction {
                db_helper::regret_everything().await?;
            }
            return Err(e);
        }
    }

    end_transaction_if_i_own_the_transaction!(do_i_own_the_transaction);

    Ok(())
}