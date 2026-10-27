use std::collections::HashMap;

use client_table_blueprints::{
    tbl_client_love_letter, tbl_every_block_in_existence, tbl_key_value_storage, tbl_pages,
    tbl_seen_server_letters,
};

use error_stuff::CqrsErr;
use leptos::prelude::*;
use leptos::logging::log;

use crate::db::cached_ids;
use crate::leptos_components::small_components::popup::create_popup;
use protocol::{new_table::ColumnDef, payload::*};
use web_internal_db;

pub async fn create_all_tables() {
    let all_defs = vec![
        tbl_client_love_letter::new_table_client_love_letter(),
        tbl_every_block_in_existence::new_table_every_block_in_existence(),
        tbl_key_value_storage::new_table_key_value_storage(),
        tbl_pages::new_table_pages(),
        tbl_seen_server_letters::new_table_seen_server_letters(),
    ];

    let mut name_map: HashMap<u8, String> = HashMap::new();
    name_map.insert(0, tbl_client_love_letter::TABLE_NAME.into());
    name_map.insert(1, tbl_every_block_in_existence::TABLE_NAME.into());
    name_map.insert(2, tbl_key_value_storage::TABLE_NAME.into());
    name_map.insert(3, tbl_pages::TABLE_NAME.into());
    name_map.insert(4, tbl_seen_server_letters::TABLE_NAME.into());

    //my db throws an err if table already exists. aka. nothing happens.
    // so ill just swallow the err.
    //
    let _ = create_each_table_one_by_one(all_defs, name_map).await;
}

pub async fn init_all() {
    create_all_tables().await;

    if let Err(e) = init_session_id().await {
        log!("init session_id failed: {:?}", e);
        create_popup(format!("init session_id failed: {e:?}"));
    }

    if let Err(e) = init_user_id().await {
        log!("init user_id failed: {:?}", e);
        create_popup(format!("init user_id failed: {e:?}"));
    }
}

const SESSION_ID_KEY: &str = "session_id";
const USER_ID_KEY: &str = "user_id";

async fn init_session_id() -> Result<(), CqrsErr> {
    let existing =
        crate::db::key_value_storage_db_actions::read_key_value_from_db(SESSION_ID_KEY).await?;
    let next = match existing {
        Some(s) => s.parse::<u64>().map(|n| n + 1).unwrap_or(1),
        None => 0,
    };
    let next_str = next.to_string();
    crate::db::key_value_storage_db_actions::write_key_value_to_db(SESSION_ID_KEY, &next_str)
        .await?;
    cached_ids::SESSION_ID.set(Some(next_str));
    Ok(())
}

async fn init_user_id() -> Result<(), CqrsErr> {
    let existing =
        crate::db::key_value_storage_db_actions::read_key_value_from_db(USER_ID_KEY).await?;
    cached_ids::USER_ID.set(existing);
    Ok(())
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
            table_name: table_name.clone(),
            columns: table_to_make,
        };
        let result = web_internal_db::db_helper::create_table(crate_table_in).await;
        if let Err(e) = result {
            log!("result: {:?}", e);
            if !e.to_string().contains("Table already exists") {
                create_popup(format!("create table {table_name}: {e}"));
            }
        }

        ctr = ctr + 1;
    }
}
