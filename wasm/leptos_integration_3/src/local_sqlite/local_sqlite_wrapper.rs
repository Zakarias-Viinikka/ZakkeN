use js_sys::Array;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen]
extern "C" {
    fn javascript_im_begging_you(args: &JsValue) -> js_sys::Promise;
}

async fn beg_js_to_work_the_worker(args: Vec<JsValue>) -> Result<JsValue, JsValue> {
    let arr = Array::new();
    for arg in &args {
        arr.push(arg);
    }
    let promise = javascript_im_begging_you(&arr);
    let result = JsFuture::from(promise).await?;
    Ok(result)
}

//list table names
pub async fn list_tables() -> Result<Vec<String>, JsValue> {
    let raw = beg_js_to_work_the_worker(build_list_tables_msg()).await?;
    let outer: Array = raw.dyn_into()?;
    let data = outer.get(1); // unwrap [command, data]

    let inner: Array = data.dyn_into()?;
    inner
        .iter()
        .map(|v| {
            v.as_string()
                .ok_or_else(|| JsValue::from_str("expected string in table list"))
        })
        .collect()
}

fn build_list_tables_msg() -> Vec<JsValue> {
    vec![JsValue::from_str("list_tables")]
}
//list table names

//create table
pub struct CreateTableColumnDef {
    pub name: String,
    pub col_type: String,
    pub primary_key: bool,
    pub not_null: bool,
    pub unique: bool,
    pub default_value: String,
    pub autoincrement: bool,
}

fn column_def_to_js(col: &CreateTableColumnDef) -> Array {
    let arr = Array::new();
    arr.push(&JsValue::from_str(&col.name));
    arr.push(&JsValue::from_str(&col.col_type));
    arr.push(&JsValue::from_bool(col.primary_key));
    arr.push(&JsValue::from_bool(col.not_null));
    arr.push(&JsValue::from_bool(col.unique));
    arr.push(&JsValue::from_str(&col.default_value));
    arr.push(&JsValue::from_bool(col.autoincrement));
    arr
}

fn build_create_table_msg(table_name: &str, columns: &[CreateTableColumnDef]) -> Vec<JsValue> {
    let cols_arr = Array::new();
    for col in columns {
        cols_arr.push(&column_def_to_js(col));
    }

    vec![
        JsValue::from_str("create_table"),
        JsValue::from_str(table_name),
        cols_arr.into(),
    ]
}

pub async fn create_table(
    table_name: &str,
    columns: &[CreateTableColumnDef],
) -> Result<String, JsValue> {
    let raw = beg_js_to_work_the_worker(build_create_table_msg(table_name, columns)).await?;
    let outer: Array = raw.dyn_into()?;
    outer
        .get(1)
        .as_string()
        .ok_or_else(|| JsValue::from_str("expected string response from create_table"))
}
//create table

//insert into table
fn build_insert_data_msg(table_name: &str, col_names: &[String], vals: &[String]) -> Vec<JsValue> {
    let cols_arr = Array::new();
    for c in col_names {
        cols_arr.push(&JsValue::from_str(c));
    }

    let vals_arr = Array::new();
    for v in vals {
        vals_arr.push(&JsValue::from_str(v));
    }

    vec![
        JsValue::from_str("insert_data"),
        JsValue::from_str(table_name),
        cols_arr.into(),
        vals_arr.into(),
    ]
}

pub async fn insert_data(
    table_name: &str,
    col_names: &[String],
    vals: &[String],
) -> Result<(), JsValue> {
    beg_js_to_work_the_worker(build_insert_data_msg(table_name, col_names, vals)).await?;
    Ok(())
}
//insert into table

//"check_table"
#[derive(Clone)]
pub struct TableColumnInfo {
    pub name: String,
    pub col_type: String,
    pub primary_key: bool,
}

fn build_check_table_msg(table_name: &str) -> Vec<JsValue> {
    vec![
        JsValue::from_str("check_table"),
        JsValue::from_str(table_name),
    ]
}

fn parse_column_info(info: &str) -> Result<TableColumnInfo, JsValue> {
    // strip the leading "infoN: " label
    let rest = info
        .split_once(':')
        .map(|(_, rest)| rest.trim())
        .unwrap_or(info);

    let mut name = None;
    let mut col_type = None;
    let mut primary_key = None;

    for part in rest.split(',') {
        let part = part.trim();
        if let Some(v) = part.strip_prefix("name=") {
            name = Some(v.to_string());
        } else if let Some(v) = part.strip_prefix("type=") {
            col_type = Some(v.to_string());
        } else if let Some(v) = part.strip_prefix("primary_key=") {
            primary_key = Some(v == "true");
        }
    }

    Ok(TableColumnInfo {
        name: name.ok_or_else(|| JsValue::from_str(&format!("missing name in: {info}")))?,
        col_type: col_type.ok_or_else(|| JsValue::from_str(&format!("missing type in: {info}")))?,
        primary_key: primary_key
            .ok_or_else(|| JsValue::from_str(&format!("missing primary_key in: {info}")))?,
    })
}

pub async fn check_table(table_name: &str) -> Result<Vec<TableColumnInfo>, JsValue> {
    let raw = beg_js_to_work_the_worker(build_check_table_msg(table_name)).await?;
    let outer: Array = raw.dyn_into()?;
    let data: Array = outer.get(1).dyn_into()?;

    data.iter()
        .map(|v| {
            let s = v
                .as_string()
                .ok_or_else(|| JsValue::from_str("expected string in check_table response"))?;
            parse_column_info(&s)
        })
        .collect()
}
//"check table"

//create index
fn build_create_index_msg(table_name: &str, column_name: &str) -> Vec<JsValue> {
    vec![
        JsValue::from_str("create_index"),
        JsValue::from_str(table_name),
        JsValue::from_str(column_name),
    ]
}

pub async fn create_index(table_name: &str, column_name: &str) -> Result<String, JsValue> {
    let raw = beg_js_to_work_the_worker(build_create_index_msg(table_name, column_name)).await?;
    let outer: Array = raw.dyn_into()?;
    outer
        .get(1)
        .as_string()
        .ok_or_else(|| JsValue::from_str("expected string response from create_index"))
}
//create index

//get data
fn build_get_data_msg(
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
//get data

//delete table
fn build_delete_table_msg(table_name: &str) -> Vec<JsValue> {
    vec![
        JsValue::from_str("delete_table"),
        JsValue::from_str(table_name),
    ]
}

pub async fn delete_table(table_name: &str) -> Result<String, JsValue> {
    let raw = beg_js_to_work_the_worker(build_delete_table_msg(table_name)).await?;
    let outer: Array = raw.dyn_into()?;
    outer
        .get(1)
        .as_string()
        .ok_or_else(|| JsValue::from_str("expected string response from delete_table"))
}
//delete table

//delete row
fn build_delete_row_msg(table_name: &str, row_id: &str) -> Vec<JsValue> {
    vec![
        JsValue::from_str("delete_row"),
        JsValue::from_str(table_name),
        JsValue::from_str(row_id),
    ]
}

pub async fn delete_row(table_name: &str, row_id: &str) -> Result<(), JsValue> {
    beg_js_to_work_the_worker(build_delete_row_msg(table_name, row_id)).await?;
    Ok(())
}
//delete row

//edit row
fn build_edit_row_msg(
    table_name: &str,
    row_id: &str,
    column: &str,
    new_value: &str,
) -> Vec<JsValue> {
    vec![
        JsValue::from_str("edit_row"),
        JsValue::from_str(table_name),
        JsValue::from_str(row_id),
        JsValue::from_str(column),
        JsValue::from_str(new_value),
    ]
}

pub async fn edit_row(
    table_name: &str,
    row_id: &str,
    column: &str,
    new_value: &str,
) -> Result<(), JsValue> {
    beg_js_to_work_the_worker(build_edit_row_msg(table_name, row_id, column, new_value)).await?;
    Ok(())
}
//edit row

//swap columns
fn build_swap_columns_msg(
    table_name: &str,
    row_id_1: &str,
    row_id_2: &str,
    column: &str,
) -> Vec<JsValue> {
    vec![
        JsValue::from_str("swap_columns"),
        JsValue::from_str(table_name),
        JsValue::from_str(row_id_1),
        JsValue::from_str(row_id_2),
        JsValue::from_str(column),
    ]
}

pub async fn swap_columns(
    table_name: &str,
    row_id_1: &str,
    row_id_2: &str,
    column: &str,
) -> Result<(), JsValue> {
    beg_js_to_work_the_worker(build_swap_columns_msg(
        table_name, row_id_1, row_id_2, column,
    ))
    .await?;
    Ok(())
}
//swap columns
