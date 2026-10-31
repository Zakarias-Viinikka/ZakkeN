use data_builder_for_operations_that_need_to_be_correct::{
    create_page, disable_page as build_disable_page,
};
use error_stuff::CqrsErr;
use executor_of_what_the_builder_built_because_the_builder_shouldnt_touch_the_db::page_executor::{
    disabled_page_requires_three_db_actions, insert_page_requires_three_db_inserts,
};

use crate::db::batcher::{run_exclusive, BatcherHandle};
use crate::db::cached_ids::{current_session_id, current_user_id};
use crate::db::main_send_unsent_love_letters::main_send_unsent_love_letters;
use crate::db::reads_wrapper::pages::{get_active_pages_blob, get_page_is_disabled, get_yrs_unblobbed};
use crate::db::writes::edit_title_for_page;
use crate::get_time;

pub async fn create_new_page(batcher: BatcherHandle) -> Result<(), CqrsErr> {
    run_exclusive(&batcher, || async move {
        let session_id = current_session_id();
        let everything = create_page(true, session_id.clone())?;
        let title_block_id = everything
            .blocks_to_insert
            .title_block
            .my_id_as_given_by_yrs
            .clone();
        let page_id = everything.page_to_insert.page_id.clone();
        insert_page_requires_three_db_inserts(everything).await?;

        let yrs = get_yrs_unblobbed(page_id, current_user_id(), get_time()).await?;
        edit_title_for_page(yrs, title_block_id, "title".to_string(), session_id).await?;
        Ok(())
    })
    .await?;
    main_send_unsent_love_letters();
    Ok(())
}

pub async fn disable_page(
    batcher: BatcherHandle,
    page_id: String,
    session_id: String,
) -> Result<(), CqrsErr> {
    let should_kick = run_exclusive(&batcher, || async move {
        if get_page_is_disabled(page_id.clone()).await? {
            return Ok(false);
        }
        let active_pages_blob = get_active_pages_blob(page_id.clone()).await?;
        let everything = build_disable_page(page_id, session_id, active_pages_blob)?;
        disabled_page_requires_three_db_actions(everything).await?;
        Ok(true)
    })
    .await?;
    if should_kick {
        main_send_unsent_love_letters();
    }
    Ok(())
}