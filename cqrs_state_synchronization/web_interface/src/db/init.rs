use std::collections::HashMap;

use client_table_blueprints::{
    new_row_helper, tbl_every_block_in_existence, tbl_pages, tbl_uncommitted_diffs,
};

use leptos::logging::log;
use protocol::{new_table::ColumnDef, payload::*};
use web_internal_db;

pub const PAGE_TABLE_NAME: &str = "pages";
pub const UNCOMMITTED_DIFFS_TABLE_NAME: &str = "uncommitted_diffs";
pub const EVERY_BLOCK_IN_EXISTENCE_TABLE_NAME: &str = "every_block_in_existence";

pub async fn create_all_tables() {
    let all_defs = vec![
        tbl_uncommitted_diffs::uncommitted_diffs_columns(),
        tbl_every_block_in_existence::every_block_in_existence_columns(),
        tbl_pages::pages_columns(),
    ];

    let mut name_map: HashMap<u8, String> = HashMap::new();
    name_map.insert(0, UNCOMMITTED_DIFFS_TABLE_NAME.into());
    name_map.insert(1, EVERY_BLOCK_IN_EXISTENCE_TABLE_NAME.into());
    name_map.insert(2, PAGE_TABLE_NAME.into());

    //my db throws an err if table already exists. aka. nothing happens.
    // so ill just swallow the err.
    //
    let _ = create_each_table_one_by_one(all_defs, name_map).await;
}

async fn create_each_table_one_by_one(
    list_of_defs_for_making_the_tables: Vec<Vec<ColumnDef>>,
    name_map: HashMap<u8, String>,
) {
    let mut ctr = 0;
    for table_to_make in list_of_defs_for_making_the_tables.into_iter() {
        /*table_name
        columns */
        let table_name = name_map.get(&ctr).unwrap().clone();
        let crate_table_in = CreateTableIn {
            table_name,
            columns: table_to_make,
        };
        let result = web_internal_db::db_helper::create_table(crate_table_in).await;
        if result.is_err() {
            log!("result: {:?}", result);
        }

        ctr = ctr + 1;
    }
}
