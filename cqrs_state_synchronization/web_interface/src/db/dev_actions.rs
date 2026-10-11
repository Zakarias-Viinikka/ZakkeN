use error_stuff::CqrsErr;
use executor_of_what_the_builder_built_because_the_builder_shouldnt_touch_the_db::{
    end_transaction_if_i_own_the_transaction, start_transaction_if_not_in_one,
};
use web_internal_db::db_helper;

use crate::db::ui_actions::create_new_page;

pub async fn insert_three_pages() -> Result<(), CqrsErr> {
    let do_i_own_the_transaction = start_transaction_if_not_in_one!();

    for _ in 0..3 {
        if let Err(e) = create_new_page().await {
            if do_i_own_the_transaction {
                db_helper::regret_everything().await?;
            }
            return Err(e);
        }
    }

    end_transaction_if_i_own_the_transaction!(do_i_own_the_transaction);

    Ok(())
}
