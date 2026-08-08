use crate::local_sqlite::column_helper;
use crate::local_sqlite::local_sqlite_wrapper;
use anyhow::{Result, anyhow};
use leptos::logging::log;
use wasm_bindgen::JsValue;

pub async fn create_table_if_not_exist() -> Result<(), JsValue> {
    /*let msg_del = local_sqlite_wrapper::delete_table("text_blocks").await?;
    log!("{}", msg_del);*/

    let tables = local_sqlite_wrapper::list_tables().await?;
    if tables.iter().any(|t| t == "text_blocks") {
        log!("text_blocks table already exists");
        return Ok(());
    }

    let columns = new_text_block_columns();

    let msg = local_sqlite_wrapper::create_table("text_blocks", &columns).await?;
    log!("{}", msg);
    Ok(())
}

pub async fn create_hardcoded_columns_if_not_exist() -> Result<()> {
    let table_name = "text_blocks";
    let arguments = "position = content";
    let columns_to_read = vec!["".to_string()]; //returns all of them
    let result = local_sqlite_wrapper::get_data(table_name, arguments, &columns_to_read).await;
    match result {
        Ok(result) => {
            let mut position_ctr = 0;
            if result.into_iter().next().is_none() {
                let column_names = vec![
                    "position".to_string(),
                    "content".to_string(),
                    "metadata".to_string(),
                ];
                for _ in 0..5 {
                    let column_values =
                        vec![position_ctr.to_string(), "".to_string(), "".to_string()];
                    position_ctr += 1;
                    local_sqlite_wrapper::insert_data(table_name, &column_names, &column_values)
                        .await
                        .map_err(|e| anyhow!(format! {"{:?}", e}))?;
                }
                log!("created 5 columns");
                Ok(())
            } else {
                log!("columns exist in db already");
                Ok(())
            }
        }
        Err(e) => Err(anyhow!(format!("{:?}", e))),
    }
}

fn new_text_block_columns() -> Vec<local_sqlite_wrapper::CreateTableColumnDef> {
    vec![
        column_helper::id_column(),
        column_helper::column("position", "INTEGER"),
        column_helper::column("content", "TEXT"),
        column_helper::column("metadata", "TEXT"),
    ]
}
