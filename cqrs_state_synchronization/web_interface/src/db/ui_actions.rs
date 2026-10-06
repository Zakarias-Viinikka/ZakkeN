use std::collections::HashMap;

use client_table_blueprints::{tbl_every_block_in_existence, tbl_pages, tbl_uncommitted_diffs};
use data_builder_for_operations_that_need_to_be_correct::create_page;
use data_builder_for_operations_that_need_to_be_correct::page;
use error_stuff::CqrsErr;
use executor_of_what_the_builder_built_because_the_builder_shouldnt_touch_the_db::page_executor::{
    disabled_page_requires_three_db_actions, insert_page_requires_three_db_inserts,
};
use leptos::{logging::log, prelude::*, reactive::spawn_local};
use protocol::{error::DbError, payload::*, schema_helper::DestructDbReturnCol};
use web_internal_db::db_helper;

use crate::{
    checkbox_logic::{self, Checkbox, CheckboxToCallbackMap},
    db::db_helpers_for_web_client::{self},
    leptos_components::small_components::popup::create_popup,
    shared_structs::LocalPages,
};

pub async fn create_new_page(
    page_ctr: RwSignal<usize>,
    local_pages: WriteSignal<Vec<LocalPages>>,
) -> Result<(), CqrsErr> {
    let session_id = crate::FAKE_SESSION_ID.to_string();
    let ctr = page_ctr.get_untracked();
    page_ctr.update(|ctr| *ctr += 1);

    let everything = create_page(true, session_id.clone())?;

    let title_block_id = everything
        .blocks_to_insert
        .title_block
        .my_id_as_given_by_yrs
        .clone();
    let page_id = everything.page_to_insert.page_id.clone();

    insert_page_requires_three_db_inserts(everything).await?;

    let new_title = format!("Title {}", ctr);

    local_pages.update(|pages| {
        pages.push(LocalPages {
            title: new_title.clone(),
            id: ctr as usize,
            yrs_id: page_id.clone(),
            is_disabled: false,
        });
    });

    let yrs = db_helpers_for_web_client::get_yrs_unblobbed(
        page_id,
        crate::FAKE_USER_ID.to_string(),
        crate::FAKE_TIME.to_string(),
    )
    .await?;

    db_helpers_for_web_client::edit_title_for_page(
        yrs,
        title_block_id,
        new_title,
        crate::FAKE_SESSION_ID.to_string(),
    )
    .await?;

    Ok(())
}

pub async fn disable_page(page_id: String, session_id: String) -> Result<(), CqrsErr> {
    let existing = db_helper::get_single_col(GetSingleColIn {
        table_name: tbl_every_block_in_existence::TABLE_NAME.into(),
        arguments: SelectArguments::Single(SelectArgument::XEqualY {
            x: tbl_every_block_in_existence::ID_OF_PAGE_I_BELONG_TO.name.to_string(),
            y: page_id.clone(),
        }),
        column_to_read: tbl_every_block_in_existence::PAGE_IS_DISABLED.name.to_string(),
    })
    .await?;

    let is_disabled = tbl_every_block_in_existence::PAGE_IS_DISABLED
        .try_destruct_db_col(existing.value)
        .map_err(|e| CqrsErr::DbErrorContainer(DbError::IllegalInput(e)))?
        .ok_or_else(|| {
            CqrsErr::DbErrorContainer(DbError::IllegalInput(
                "page_is_disabled was null".to_string(),
            ))
        })?;

    if is_disabled == "true" {
        return Ok(());
    }

    let out = db_helper::get_single_col(GetSingleColIn {
        table_name: tbl_pages::TABLE_NAME.into(),
        arguments: SelectArguments::Single(SelectArgument::XEqualY {
            x: tbl_pages::PAGE_ID.name.to_string(),
            y: page_id.clone(),
        }),
        column_to_read: tbl_pages::PAGE_STATUS.name.to_string(),
    })
    .await?;

    let active_pages_blob = tbl_pages::PAGE_STATUS
        .try_destruct_db_col(out.value)
        .map_err(|e| CqrsErr::DbErrorContainer(DbError::IllegalInput(e)))?
        .ok_or_else(|| {
            CqrsErr::DbErrorContainer(DbError::IllegalInput(
                "db doesn't stop us from asking for the col for a row that doesn't exist".to_string(),
            ))
        })?;

    let everything = page::disable_page(page_id, session_id, active_pages_blob)?;

    disabled_page_requires_three_db_actions(everything).await?;

    Ok(())
}

pub fn delete_everything(ctr: RwSignal<usize>, local_pages_set: WriteSignal<Vec<LocalPages>>) {
    ctr.set(0);
    local_pages_set.set(Vec::new());

    let tables_to_delete = vec![
        tbl_pages::TABLE_NAME,
        tbl_every_block_in_existence::TABLE_NAME,
        tbl_uncommitted_diffs::TABLE_NAME,
    ];

    for table_name in &tables_to_delete {
        do_delete_all_rows(table_name);
    }
}

fn do_delete_all_rows(table_name: &str) {
    let delete_all_rows_in = DeleteAllRowsIn {
        table_name: table_name.into(),
    };

    spawn_local(async move {
        let result = db_helper::delete_all_rows(delete_all_rows_in).await;
        alert_if_error(result);
    });
}

fn alert_if_error<T, E: std::fmt::Debug>(result: Result<T, E>) {
    if let Err(e) = result {
        log!("Error: {:?}", e);
        create_popup(format!("Error: {:?}", e));
    }
}

/*
* pub struct CheckboxToCallbackMap {
    pub map: HashMap<String, Box<dyn Fn() + Send + Sync>>,
}

#[derive(Clone)]
pub struct AllCheckboxes {
    pub checkboxes: RwSignal<HashMap<String, RwSignal<Checkbox>>>,
}
*/

pub async fn run_all_seeds(
    checkboxes: RwSignal<HashMap<String, RwSignal<Checkbox>>>,
    map_of_callbacks: &CheckboxToCallbackMap,
) {
    let map_of_callbacks = &map_of_callbacks.map;
    let checkboxes = checkboxes.get_untracked();

    let mut callbacks_to_run: Vec<&checkbox_logic::AsyncCallback> = vec![];

    for (key, checkbox) in checkboxes.into_iter() {
        let callback = map_of_callbacks
            .get(&key)
            .expect("no callback for checkbox");

        if checkbox.with_untracked(|c| c.is_active.get_untracked()) {
            callbacks_to_run.push(callback);
        }
    }

    for callback in callbacks_to_run {
        callback().await;
    }

    create_popup("finished seeding".into());
}
