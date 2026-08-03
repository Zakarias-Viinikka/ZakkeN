use crate::local_sqlite::column_helper;
use crate::local_sqlite::local_sqlite_wrapper;
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

    let columns = [
        column_helper::id_column(),
        column_helper::column("position", "INTEGER"),
        column_helper::column("content", "TEXT"),
        column_helper::column("metadata", "TEXT"),
    ];

    let msg = local_sqlite_wrapper::create_table("text_blocks", &columns).await?;
    log!("{}", msg);
    Ok(())
}

/*
* fn build_get_data_msg(
    table_name: &str,
    arguments: &str,
    columns_to_read: &[String],
) -> Vec<JsValue> {
    let cols_arr = Array::new();
    for c in columns_to_read {
        cols_arr.push(&JsValue::from_str(c));
    }
    vec![
        JsValue::from_str("get_data"),
        JsValue::from_str(table_name),
        JsValue::from_str(arguments),
        cols_arr.into(),
    ]
}

pub async fn get_data(
    table_name: &str,
    arguments: &str,
    columns_to_read: &[String],
) -> Result<Vec<Vec<String>>, JsValue> {
    let raw = beg_js_to_work_the_worker(build_get_data_msg(table_name, arguments, columns_to_read))
        .await?;
    let outer: Array = raw.dyn_into()?;
    let rows: Array = outer.get(1).dyn_into()?;

    rows.iter()
        .map(|row_val| {
            let row: Array = row_val.dyn_into()?;
            row.iter()
                .map(|cell| {
                    cell.as_string().ok_or_else(|| {
                        JsValue::from_str("expected string cell in get_data response")
                    })
                })
                .collect()
        })
        .collect()
}
*/
