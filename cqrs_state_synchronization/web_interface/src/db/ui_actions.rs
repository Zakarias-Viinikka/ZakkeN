use std::collections::HashMap;

use client_table_blueprints::{tbl_every_block_in_existence, tbl_pages, tbl_uncommitted_diffs};
use data_builder_for_operations_that_need_to_be_correct::create_page;
use executor_of_what_the_builder_built_because_the_builder_shouldnt_touch_the_db::page_executor::insert_page_requires_three_db_inserts;
use leptos::{logging::log, prelude::*, reactive::spawn_local};
use web_internal_db::db_helper;

use crate::{
    checkbox_logic::{Checkbox, CheckboxToCallbackMap},
    db::db_helpers_for_web_client::{self},
    leptos_components::small_components::popup::create_popup,
    shared_structs::LocalPages,
};

pub fn create_new_page(page_ctr: RwSignal<usize>, local_pages: WriteSignal<Vec<LocalPages>>) {
    let session_id = crate::FAKE_SESSION_ID.to_string();
    let ctr = page_ctr.get();
    page_ctr.update(|ctr| *ctr += 1);
    spawn_local(async move {
        let everything = match create_page(true, session_id.clone()) {
            Ok(v) => v,
            Err(e) => {
                log!("create_page failed: {:?}", e);
                return;
            }
        };
        let title_block_id = everything
            .blocks_to_insert
            .title_block
            .my_id_as_given_by_yrs
            .clone();
        let page_id = everything.page_to_insert.page_id.clone();
        if let Err(e) = insert_page_requires_three_db_inserts(everything).await {
            log!("insert failed: {:?}", e);
        }

        let new_title = format!("Title {}", ctr);

        local_pages.update(|pages| {
            pages.push(LocalPages {
                title: new_title.clone(),
                id: ctr as usize,
                yrs_id: page_id.clone(),
            });
        });

        let yrs = match db_helpers_for_web_client::get_yrs_unblobbed(
            page_id,
            crate::FAKE_USER_ID.to_string(),
            crate::FAKE_TIME.to_string(),
        )
        .await
        {
            Ok(v) => v,
            Err(e) => {
                log!("get_yrs_unblobbed failed: {:?}", e);
                create_popup(format!("Error: {:?}", e));
                return;
            }
        };

        let result = db_helpers_for_web_client::edit_title_for_page(
            yrs,
            title_block_id,
            new_title,
            crate::FAKE_SESSION_ID.to_string(),
        )
        .await;

        match result {
            Err(e) => {
                log!("edit_title_for_page failed: {:?}", e);
                create_popup(format!("Error: {:?}", e));
            }
            _ => {}
        }
    });

    crate::leptos_components::small_components::popup::create_popup("Created Page".into());
}

pub fn delete_everything() {
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
    let mut delete_all_rows_in = DeleteAllRowsIn { table_name };

    let result = db_helper::delete_all_rows(delete_all_rows_in);
    alert_if_error(result);
}

fn alert_if_error(result: Result<_, Error>) {
    if let Err(e) = result {
        log!("Error: {:?}", e);
        crate::leptos_components::small_components::popup::create_popup(format!("Error: {:?}", e));
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

pub fn run_all_seeds(
    checkboxes: RwSignal<HashMap<String, RwSignal<Checkbox>>>,
    map_of_callbacks: CheckboxToCallbackMap,
) {
    let checkboxes = checkboxes.get();
    let map_of_callbacks = map_of_callbacks.map;

    let callbacks_to_run = vec![];

    for checkbox in checkboxes.into_iter() {
        let key = checkbox.0;
        let checkbox = checkbox.1;
        let callback = map_of_callbacks
            .get(&key)
            .cloned()
            .expect("no callback for checkbox");

        if checkbox.is_active.get() {
            callbacks_to_run.push(callback);
        }
    }

    for callback in callbacks_to_run {
        callback();
    }
}
