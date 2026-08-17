//use js_sys::{Array, Uint8Array};
use wasm_bindgen::JsValue;
// new for sahpool
use sqlite_wasm_rs as ffi;
use sqlite_wasm_vfs::sahpool::{OpfsSAHPoolCfg, OpfsSAHPoolUtil, install as install_opfs_sahpool};

//use crate::black_magic_read::read_from_db;
use crate::create_sql_statements::*;
//use crate::db_table::*;

use anyhow::{Result, anyhow, bail};

use crate::create_table::ColumnDef;
use crate::public_data_shapes::DbError;

pub async fn create_local_db_connection(
    conn_name: &str,
) -> Result<(OpfsSAHPoolUtil, rusqlite::Connection)> {
    let sahpool_util =
        install_opfs_sahpool::<ffi::WasmOsCallback>(&OpfsSAHPoolCfg::default(), true).await?;

    let conn = rusqlite::Connection::open_with_flags(
        conn_name,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE | rusqlite::OpenFlags::SQLITE_OPEN_CREATE,
    )?;

    Ok((sahpool_util, conn))
}

pub fn create_table(
    conn: &rusqlite::Connection,
    table_name: &str,
    columns: Vec<ColumnDef>,
) -> Result<(), DbError> {
    if table_name.is_empty() {
        return Err(DbError::IllegalInput("table_name is empty".to_string()));
    }
    let sql = generate_create_table_sql(table_name, &columns);
    conn.execute(&sql, [])
        .map_err(|e| DbError::SqlExecuteFail(format!("err: {}, sql: {}", e, sql)))?;
    Ok(())
}

pub fn close_conn(conn: rusqlite::Connection) -> Result<(), DbError> {
    conn.close()
        .map_err(|(_, e)| DbError::ConnError(format!("Failed to close connection: {}", e)))?;
    Ok(())
}

pub fn list_tables(conn: &rusqlite::Connection) -> Result<Vec<String>, DbError> {
    let sql = generate_read_from_table_sql(
        "sqlite_master",
        &["type = 'table'", "name NOT LIKE 'sqlite_%'"],
        &["name"],
    );

    let result = conn.prepare(&sql);
    let Ok(mut stmt) = result else {
        return Err(DbError::SqlExecuteFail(format!(
            "failed to execute prepare when trying to list tables: {:?}, sql: {}",
            result, sql
        )));
    };

    let tables = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| DbError::BadCode(format!("failed to query list tables: {}", e)))?
        .collect::<Result<Vec<String>, _>>()
        .map_err(|e| DbError::BadCode(format!("failed to query list tables: {}", e)))?;

    Ok(tables)
}

pub fn insert_into_table(
    conn: &rusqlite::Connection,
    table_name: &str,
    values: Vec<(String, String)>,
) -> Result<()> {
    let sql = generate_insert_sql(table_name, values);
    conn.execute(&sql, [])?;
    Ok(())
}

pub fn drop_table(conn: &rusqlite::Connection, table_name: &str) -> Result<(), JsValue> {
    let sql = format!("DROP TABLE IF EXISTS {};", table_name);
    conn.execute(&sql, [])
        .map_err(|e| JsValue::from(e.to_string()))?;
    Ok(())
}

pub fn edit_col_in_row(
    conn: &rusqlite::Connection,
    table_name: &str,
    row: &str,
    column_and_new_value: (impl AsRef<str>, impl AsRef<str>),
) -> Result<()> {
    let id: usize = row.parse()?;
    let sql = generate_update_sql(
        table_name,
        id,
        &(
            column_and_new_value.0.as_ref(),
            column_and_new_value.1.as_ref(),
        ),
    );
    conn.execute(&sql, [])?;
    Ok(())
}

pub fn delete_row(conn: &rusqlite::Connection, table_name: &str, row_id: &str) -> Result<()> {
    let sql = generate_delete_sql(table_name, row_id);
    conn.execute(&sql, [])?;
    Ok(())
}

pub fn create_index(
    conn: &rusqlite::Connection,
    table_name: &str,
    column_name: &str,
) -> Result<()> {
    let sql = format!(
        "CREATE INDEX IF NOT EXISTS idx_{}_{} ON {}({});",
        table_name, column_name, table_name, column_name
    );
    conn.execute(&sql, [])?;
    Ok(())
}

pub fn table_shape(conn: &rusqlite::Connection, table_name: &str) -> Result<Vec<String>, JsValue> {
    let sql = format!("PRAGMA table_info({})", table_name);
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| JsValue::from(e.to_string()))?;

    let columns = stmt
        .query_map([], |row| {
            let cid: i64 = row.get(0)?;
            let name: String = row.get(1)?;
            let col_type: String = row.get(2)?;
            let not_null: i64 = row.get(3)?;
            let pk: i64 = row.get(5)?;
            Ok(format!(
                "info{}: name={}, type={}, not_null={}, primary_key={}",
                cid,
                name,
                col_type,
                not_null != 0,
                pk != 0
            ))
        })
        .map_err(|e| JsValue::from(e.to_string()))?
        .collect::<Result<Vec<String>, _>>()
        .map_err(|e| JsValue::from(e.to_string()))?;

    Ok(columns)
}
