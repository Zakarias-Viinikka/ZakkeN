use crate::local_sqlite::column_helper;
use crate::local_sqlite::local_sqlite_wrapper;
use anyhow::{Result, anyhow, bail};
use leptos::logging::log;
use leptos::prelude::*;
use serde::de::Unexpected::NewtypeStruct;
use text_diff2::text_block::TextBlock;
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
    let arguments = "";
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

pub async fn update_local_sqlite(text_block: TextBlock, new_text: String) -> Result<()> {
    let table_name = "text_blocks";
    let row_id = get_row_id_for_text_block(&text_block, table_name).await?;

    local_sqlite_wrapper::edit_row(table_name, &row_id, "content", &new_text)
        .await
        .map_err(|e| anyhow!("{:?}", e))?;

    Ok(())
}

async fn get_row_id_for_text_block(text_block: &TextBlock, table_name: &str) -> Result<String> {
    // Query the database for the row whose "position" matches the text_block's position
    let argument = format!("position = {}", text_block.id.get_untracked());
    let rows: Vec<Vec<String>> = local_sqlite_wrapper::get_data(
        table_name,
        &argument,
        &["id".to_string()], // read only the "id" column
    )
    .await
    .map_err(|e| anyhow!("get_data failed: {:?}", e))?;

    let row = rows.into_iter().next().ok_or_else(|| {
        anyhow!(
            "no row found for position = {}",
            text_block.id.get_untracked()
        )
    })?;

    let id = row
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("row is missing the 'id' column"))?;

    Ok(id)
}

pub async fn move_row(
    text_block: &TextBlock,
    above: Option<TextBlock>,
    below: Option<TextBlock>,
    table_name: &str,
) -> Result<()> {
    let arguments = "";
    let columns_to_read = vec!["id".to_string(), "position".to_string()];
    /*let all_ids_n_positions =
    local_sqlite_wrapper::get_data(table_name, arguments, &columns_to_read)
        .await
        .map_err(|e| anyhow!("failed moving row in local db: {:?}", e))?;*/

    let id_of_row_to_move = get_row_id_for_text_block(text_block, table_name).await?;

    let new_position;
    match get_above_and_below_id(above, below) {
        (Some(above_position), Some(below_position)) => {
            new_position = above_position + below_position / 2.0;
        }
        (Some(above), None) => {
            new_position = above + 0.5;
        }
        (None, Some(below)) => {
            new_position = below - 0.5;
        }
        _ => {
            bail!("move_row: should be impossible for below and above to be none");
        }
    }

    let result = local_sqlite_wrapper::edit_row(
        table_name,
        &id_of_row_to_move,
        "position",
        &&new_position.to_string(),
    )
    .await
    .map_err(|e| anyhow!(format!("{:?}", e)))?;

    //pub async fn edit_row(table_name: &str, row_id: &str, column: &str, new_value: &str) -> Result<(), JsValue>

    Ok(result)
}

fn get_above_and_below_id(
    above: Option<TextBlock>,
    below: Option<TextBlock>,
) -> (Option<f64>, Option<f64>) {
    match (above, below) {
        (Some(above), Some(below)) => {
            let above_position = above.id.get_untracked();
            let below_position = below.id.get_untracked();
            (Some(above_position), Some(below_position))
        }
        (Some(above), None) => {
            let above_position = above.id.get_untracked();
            (Some(above_position), None)
        }
        (None, Some(below)) => {
            let below_position = below.id.get_untracked();
            (None, Some(below_position))
        }
        (None, None) => {
            log!("catastrophic error in find_above_and_below");
            (None, None)
        }
    }
}
