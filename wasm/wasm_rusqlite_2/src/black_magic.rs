// new for sahpool
use sqlite_wasm_rs as ffi;
use sqlite_wasm_vfs::sahpool::{OpfsSAHPoolCfg, OpfsSAHPoolUtil, install as install_opfs_sahpool};

//use crate::black_magic_read::read_from_db;
use crate::create_sql_statements::*;
//use crate::db_table::*;

use anyhow::Result;

use crate::DbError;
use crate::create_table::ColumnDef;
use crate::public_data_shapes::*;
use crate::table_row::*;

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
    values: Vec<ColumnValue>,
) -> Result<(), DbError> {
    let values: Vec<(String, Col)> = values
        .into_iter()
        .map(|cv| (cv.column_name, cv.value))
        .collect();

    let sql = generate_insert_sql(table_name, values);
    conn.execute(&sql, []).map_err(|e| {
        DbError::SqlExecuteFail(format!("insert_into_table failed: {}, sql: {}", e, sql))
    })?;

    Ok(())
}

pub fn drop_table(conn: &rusqlite::Connection, table_name: &str) -> Result<(), DbError> {
    let sql = format!("DROP TABLE IF EXISTS {};", quote_ident(table_name));
    conn.execute(&sql, [])
        .map_err(|e| DbError::SqlExecuteFail(format!("drop_table failed: {}, sql: {}", e, sql)))?;
    Ok(())
}

pub fn edit_col_in_row(
    conn: &rusqlite::Connection,
    table_name: &str,
    row: &str,
    column: &str,
    new_value: &Col,
) -> Result<(), DbError> {
    let id: usize = row
        .parse()
        .map_err(|e| DbError::IllegalInput(format!("row id is not a number: {}", e)))?;

    let sql = generate_update_sql_typed(table_name, id, column, new_value);

    conn.execute(&sql, []).map_err(|e| {
        DbError::SqlExecuteFail(format!("edit_col_in_row failed: {}, sql: {}", e, sql))
    })?;

    Ok(())
}

pub fn delete_row(
    conn: &rusqlite::Connection,
    table_name: &str,
    row_id: &str,
) -> Result<(), DbError> {
    let sql = generate_delete_sql(table_name, row_id);
    conn.execute(&sql, [])
        .map_err(|e| DbError::SqlExecuteFail(format!("delete_row failed: {}, sql: {}", e, sql)))?;
    Ok(())
}

pub fn create_index(
    conn: &rusqlite::Connection,
    table_name: &str,
    column_name: &str,
) -> Result<(), DbError> {
    let index_name = quote_ident(&format!("idx_{}_{}", table_name, column_name));
    let quoted_table = quote_ident(table_name);
    let quoted_column = quote_ident(column_name);

    let sql = format!(
        "CREATE INDEX IF NOT EXISTS {} ON {} ({});",
        index_name, quoted_table, quoted_column
    );

    conn.execute(&sql, []).map_err(|e| {
        DbError::SqlExecuteFail(format!("create_index failed: {}, sql: {}", e, sql))
    })?;

    Ok(())
}

pub fn check_index(
    conn: &rusqlite::Connection,
    table_name: &str,
    column_name: &str,
) -> Result<bool, DbError> {
    let index_list_sql = format!("PRAGMA index_list({})", quote_sql_string(table_name));
    let mut stmt = conn
        .prepare(&index_list_sql)
        .map_err(|e| DbError::SqlExecuteFail(e.to_string()))?;

    let index_names: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| DbError::SqlExecuteFail(e.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| DbError::SqlExecuteFail(e.to_string()))?;

    for index_name in index_names {
        let index_info_sql = format!("PRAGMA index_info({})", quote_sql_string(&index_name));
        let mut info_stmt = conn
            .prepare(&index_info_sql)
            .map_err(|e| DbError::SqlExecuteFail(e.to_string()))?;

        let indexed_columns: Vec<String> = info_stmt
            .query_map([], |row| row.get::<_, String>(2))
            .map_err(|e| DbError::SqlExecuteFail(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| DbError::SqlExecuteFail(e.to_string()))?;

        if indexed_columns
            .iter()
            .any(|c| c.eq_ignore_ascii_case(column_name))
        {
            return Ok(true);
        }
    }

    Ok(false)
}

pub fn table_shape(
    conn: &rusqlite::Connection,
    table_name: &str,
) -> Result<Vec<TableColumnInfo>, DbError> {
    let sql = format!("PRAGMA table_info({})", quote_sql_string(table_name));
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| DbError::SqlExecuteFail(e.to_string()))?;

    let columns = stmt
        .query_map([], |row| {
            let cid: i64 = row.get(0)?;
            let name: String = row.get(1)?;
            let type_name: String = row.get(2)?;
            let not_null_raw: i64 = row.get(3)?;
            let default_value: Option<String> = row.get(4)?;
            let pk_raw: i64 = row.get(5)?;

            Ok(TableColumnInfo {
                cid,
                name,
                type_name,
                not_null: not_null_raw != 0,
                default_value,
                primary_key: pk_raw != 0,
            })
        })
        .map_err(|e| DbError::SqlExecuteFail(e.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| DbError::SqlExecuteFail(e.to_string()))?;

    Ok(columns)
}

pub fn add_column(
    conn: &rusqlite::Connection,
    table_name: &str,
    column: ColumnDef,
) -> Result<(), DbError> {
    if table_name.is_empty() {
        return Err(DbError::IllegalInput("table_name is empty".to_string()));
    }

    if column.primary_key {
        return Err(DbError::IllegalInput(
            "ADD COLUMN does not support PRIMARY KEY".to_string(),
        ));
    }
    if column.autoincrement {
        return Err(DbError::IllegalInput(
            "ADD COLUMN does not support AUTOINCREMENT".to_string(),
        ));
    }
    if column.unique {
        return Err(DbError::IllegalInput(
            "ADD COLUMN does not support UNIQUE".to_string(),
        ));
    }
    if column.not_null && column.default_value.is_empty() {
        return Err(DbError::IllegalInput(
            "NOT NULL column requires a DEFAULT value when adding to an existing table".to_string(),
        ));
    }

    let sql = generate_add_column_sql(table_name, &column);
    conn.execute(&sql, [])
        .map_err(|e| DbError::SqlExecuteFail(format!("add_column failed: {}, sql: {}", e, sql)))?;

    Ok(())
}

pub fn remove_column(
    conn: &rusqlite::Connection,
    table_name: &str,
    column_name: &str,
) -> Result<(), DbError> {
    let sql = generate_drop_column_sql(table_name, column_name);
    conn.execute(&sql, []).map_err(|e| {
        DbError::SqlExecuteFail(format!("remove_column failed: {}, sql: {}", e, sql))
    })?;

    Ok(())
}

pub fn export_database(util: &OpfsSAHPoolUtil, conn_name: &str) -> Result<Vec<u8>, DbError> {
    util.export_db(conn_name)
        .map_err(|e| DbError::SqlExecuteFail(format!("export_database failed: {}", e)))
}

pub fn create_table_from_export(
    conn: &rusqlite::Connection,
    table_name: &str,
    table_export: &TableExport,
) -> Result<(), DbError> {
    let exists_sql = "SELECT 1 FROM sqlite_master WHERE type='table' AND name=? LIMIT 1";
    let mut exists_stmt = conn
        .prepare(exists_sql)
        .map_err(|e| DbError::SqlExecuteFail(e.to_string()))?;

    let table_exists = exists_stmt
        .exists([table_name])
        .map_err(|e| DbError::SqlExecuteFail(e.to_string()))?;

    if table_exists {
        return Err(DbError::IllegalInput(format!(
            "Table already exists: {}",
            table_name
        )));
    }

    let mut columns: Vec<ColumnDef> = Vec::new();
    for col in &table_export.columns {
        columns.push(ColumnDef {
            name: col.name.clone(),
            column_type: col.type_name.clone(),
            primary_key: col.primary_key,
            not_null: col.not_null,
            unique: false,
            default_value: col.default_value.clone().unwrap_or_default(),
            autoincrement: false,
        });
    }

    /*
    *

    */

    create_table(conn, table_name, columns)?;

    for row in &table_export.rows {
        let mut values = Vec::new();
        for (idx, col) in row.cols.iter().enumerate() {
            let column_name = table_export
                .columns
                .get(idx)
                .map(|c| c.name.clone())
                .unwrap_or_default();

            values.push(ColumnValue {
                column_name,
                value: col.clone(),
            });
        }
        insert_into_table(conn, table_name, values)?;
    }

    Ok(())
}

pub fn copy_table(
    conn: &rusqlite::Connection,
    source_table_name: &str,
    new_table_name: &str,
) -> Result<(), DbError> {
    let sql = format!(
        "CREATE TABLE {} AS SELECT * FROM {};",
        quote_ident(new_table_name),
        quote_ident(source_table_name)
    );

    conn.execute(&sql, [])
        .map_err(|e| DbError::SqlExecuteFail(format!("copy_table failed: {}, sql: {}", e, sql)))?;

    Ok(())
}
