//use js_sys::{Array, Uint8Array};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Blob, Url, console::log_1};
// new for sahpool
use sqlite_wasm_rs::{
    self as ffi,
    sahpool_vfs::{OpfsSAHPoolCfg, install as install_opfs_sahpool},
};
//old //use sqlite_wasm_rs as ffi; //necessary as far as i can tell.

use crate::black_magic_read::read_from_db;
use crate::create_sql_statements::*;
use crate::db_table::*;

use anyhow::{Result, anyhow, bail};
use std::ffi::CString; //let sql_cstr = CString::new(sql).map_err(|e| anyhow!("CString conversion failed: {}", e))?;

pub async fn create_local_db_connection(conn_name: &str) -> Result<*mut ffi::sqlite3> {
    let filename_cstr = CString::new(conn_name)?;
    let mut db = std::ptr::null_mut();

    // must be awaited to completion BEFORE opening
    install_opfs_sahpool(&OpfsSAHPoolCfg::default(), true).await?;
    // null = "use the default vfs", which is now sahpool since we passed `true` above
    let vfs_name: *const std::os::raw::c_char = std::ptr::null();
    let ret = unsafe {
        ffi::sqlite3_open_v2(
            filename_cstr.as_ptr().cast(),
            &mut db as *mut _,
            ffi::SQLITE_OPEN_READWRITE | ffi::SQLITE_OPEN_CREATE,
            vfs_name,
        )
    };
    if ret != ffi::SQLITE_OK {
        bail!("Failed to open database: {}", ffi::code_to_str(ret));
    }
    Ok(db)
}

// Builds a table from a caller-supplied column list — replaces the old
// fixed Table/Column version. Caller decides every column + constraint.
use crate::create_table_col_def::ColumnDef;

pub fn create_table(
    db: *mut ffi::sqlite3,
    table_name: &str,
    columns: Vec<ColumnDef>,
) -> Result<()> {
    let sql = generate_create_table_sql(table_name, &columns);
    let sql_cstr = CString::new(sql).map_err(|e| anyhow!("CString conversion failed: {}", e))?;
    unsafe {
        let ret = ffi::sqlite3_exec(
            db,
            sql_cstr.as_ptr().cast(),
            None,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
        if ret != ffi::SQLITE_OK {
            bail!("Failed to create table: {}", ffi::code_to_str(ret));
        }
        Ok(())
    }
}

// Adds an index on one column so sorting/filtering by it doesn't full-scan.
pub fn create_index(db: *mut ffi::sqlite3, table_name: &str, column_name: &str) -> Result<()> {
    let sql = format!(
        "CREATE INDEX IF NOT EXISTS idx_{}_{} ON {}({});",
        table_name, column_name, table_name, column_name
    );
    let sql_cstr = CString::new(sql).map_err(|e| anyhow!("CString conversion failed: {}", e))?;
    unsafe {
        let ret = ffi::sqlite3_exec(
            db,
            sql_cstr.as_ptr().cast(),
            None,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
        if ret != ffi::SQLITE_OK {
            bail!("Failed to create index: {}", ffi::code_to_str(ret));
        }
        Ok(())
    }
}

pub fn insert_into_table(
    db: *mut ffi::sqlite3,
    table_name: &str,
    values: Vec<(String, String)>,
) -> Result<()> {
    let sql = generate_insert_sql(table_name, values);
    web_sys::console::log_1(&format!("generated SQL: {}", sql).into());
    let sql_cstr = CString::new(sql).map_err(|e| anyhow!("CString conversion failed: {}", e))?;
    unsafe {
        let ret = ffi::sqlite3_exec(
            db,
            sql_cstr.as_ptr(),
            None,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
        if ret != ffi::SQLITE_OK {
            bail!("insert failed: {}", ffi::code_to_str(ret));
        }
    }
    Ok(())
}

pub fn drop_table(db_conn: *mut ffi::sqlite3, table_name: &str) -> Result<(), JsValue> {
    let sql = format!("DROP TABLE IF EXISTS {};", table_name);
    let c_sql = CString::new(sql).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let ret = unsafe {
        ffi::sqlite3_exec(
            db_conn,
            c_sql.as_ptr(),
            None,                 // callback: Option<fn(...)>
            std::ptr::null_mut(), // arg: *mut c_void
            std::ptr::null_mut(), // errmsg: *mut *mut c_char
        )
    };
    if ret != ffi::SQLITE_OK {
        let err = unsafe { ffi::sqlite3_errmsg(db_conn) };
        let err_str = unsafe { std::ffi::CStr::from_ptr(err).to_string_lossy().into_owned() };
        return Err(JsValue::from_str(&format!("Drop failed: {}", err_str)));
    }
    Ok(())
}

pub fn table_shape(db_conn: *mut ffi::sqlite3, table_name: &str) -> Result<Vec<String>, JsValue> {
    let sql = format!("PRAGMA table_info({});", table_name);
    let c_sql = CString::new(sql).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let mut stmt: *mut ffi::sqlite3_stmt = std::ptr::null_mut();
    // SAFETY: db_conn is a valid open connection, c_sql is a valid nul-terminated
    // string, and stmt is a valid out-pointer for prepare_v2 to write into.
    let ret = unsafe {
        ffi::sqlite3_prepare_v2(db_conn, c_sql.as_ptr(), -1, &mut stmt, std::ptr::null_mut())
    };
    if ret != ffi::SQLITE_OK {
        let err = unsafe { ffi::sqlite3_errmsg(db_conn) };
        let err_str = unsafe { std::ffi::CStr::from_ptr(err).to_string_lossy().into_owned() };
        return Err(JsValue::from_str(&format!("Prepare failed: {}", err_str)));
    }

    let mut columns = Vec::new();

    loop {
        // SAFETY: stmt was just successfully prepared above and is not yet finalized.
        let step_ret = unsafe { ffi::sqlite3_step(stmt) };
        match step_ret {
            ffi::SQLITE_ROW => {
                // SAFETY: stmt is on a valid row; column indices 0-5 match the
                // known column order of PRAGMA table_info's result set.
                unsafe {
                    let cid = ffi::sqlite3_column_int(stmt, 0);
                    let name =
                        std::ffi::CStr::from_ptr(ffi::sqlite3_column_text(stmt, 1) as *const i8)
                            .to_string_lossy()
                            .into_owned();
                    let col_type =
                        std::ffi::CStr::from_ptr(ffi::sqlite3_column_text(stmt, 2) as *const i8)
                            .to_string_lossy()
                            .into_owned();
                    let not_null = ffi::sqlite3_column_int(stmt, 3) != 0;
                    let pk = ffi::sqlite3_column_int(stmt, 5) != 0;

                    columns.push(format!(
                        "info{}: name={}, type={}, not_null={}, primary_key={}",
                        cid, name, col_type, not_null, pk
                    ));
                }
            }
            ffi::SQLITE_DONE => break,
            _ => {
                let err = unsafe { ffi::sqlite3_errmsg(db_conn) };
                let err_str =
                    unsafe { std::ffi::CStr::from_ptr(err).to_string_lossy().into_owned() };
                unsafe { ffi::sqlite3_finalize(stmt) };
                return Err(JsValue::from_str(&format!("Step failed: {}", err_str)));
            }
        }
    }

    // SAFETY: stmt is non-null and was successfully prepared; finalize is safe
    // to call exactly once when done stepping.
    unsafe { ffi::sqlite3_finalize(stmt) };

    Ok(columns)
}

pub fn edit_col_in_row(
    db: *mut ffi::sqlite3,
    table_name: &str,
    row: &str,
    column_and_new_value: (impl AsRef<str>, impl AsRef<str>),
) -> Result<()> {
    if db.is_null() {
        bail!("db pointer is null");
    }
    let (column, new_value) = (
        column_and_new_value.0.as_ref(),
        column_and_new_value.1.as_ref(),
    );
    let sql = generate_update_sql(table_name, row.parse()?, &(column, new_value));
    let sql_cstr = CString::new(sql)?;

    let mut stmt: *mut ffi::sqlite3_stmt = std::ptr::null_mut();
    let ret = unsafe {
        ffi::sqlite3_prepare_v2(db, sql_cstr.as_ptr(), -1, &mut stmt, std::ptr::null_mut())
    };
    if ret != ffi::SQLITE_OK {
        bail!(
            "prepare failed: {} (sql: {})",
            ffi::code_to_str(ret),
            generate_update_sql(table_name, row.parse()?, &(column, new_value))
        );
    }

    let value_cstr = CString::new(column)?;
    let id_cstr = CString::new(new_value)?;
    unsafe {
        ffi::sqlite3_bind_text(stmt, 1, value_cstr.as_ptr(), -1, ffi::SQLITE_TRANSIENT());
        ffi::sqlite3_bind_text(stmt, 2, id_cstr.as_ptr(), -1, ffi::SQLITE_TRANSIENT());
    }

    let step_ret = unsafe { ffi::sqlite3_step(stmt) };
    unsafe { ffi::sqlite3_finalize(stmt) };

    if step_ret != ffi::SQLITE_DONE {
        bail!("update failed: step returned {}", step_ret);
    }

    Ok(())
}

pub fn delete_row(db: *mut ffi::sqlite3, table_name: &str, row_id: &str) -> Result<()> {
    if db.is_null() {
        bail!("db pointer is null");
    }

    let sql = generate_delete_sql(table_name, row_id); //format!("DELETE FROM {table_name} WHERE {pk_col} = ?1;");
    let sql_cstr = CString::new(sql)?;

    let mut stmt: *mut ffi::sqlite3_stmt = std::ptr::null_mut();
    let ret = unsafe {
        ffi::sqlite3_prepare_v2(db, sql_cstr.as_ptr(), -1, &mut stmt, std::ptr::null_mut())
    };
    if ret != ffi::SQLITE_OK {
        bail!("prepare failed: {}", ffi::code_to_str(ret));
    }

    let id_cstr = CString::new(row_id)?;
    unsafe {
        ffi::sqlite3_bind_text(stmt, 1, id_cstr.as_ptr(), -1, ffi::SQLITE_TRANSIENT());
    }

    let step_ret = unsafe { ffi::sqlite3_step(stmt) };
    unsafe { ffi::sqlite3_finalize(stmt) };

    if step_ret != ffi::SQLITE_DONE {
        bail!("delete failed: step returned {}", step_ret);
    }

    Ok(())
}

pub fn list_tables(db_conn: *mut ffi::sqlite3) -> Result<Vec<String>, JsValue> {
    let sql = "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%';";
    let c_sql = CString::new(sql).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let mut stmt: *mut ffi::sqlite3_stmt = std::ptr::null_mut();
    // SAFETY: db_conn is a valid open connection, c_sql is a valid nul-terminated
    // string, and stmt is a valid out-pointer for prepare_v2 to write into.
    let ret = unsafe {
        ffi::sqlite3_prepare_v2(db_conn, c_sql.as_ptr(), -1, &mut stmt, std::ptr::null_mut())
    };
    if ret != ffi::SQLITE_OK {
        let err = unsafe { ffi::sqlite3_errmsg(db_conn) };
        let err_str = unsafe { std::ffi::CStr::from_ptr(err).to_string_lossy().into_owned() };
        return Err(JsValue::from_str(&format!("Prepare failed: {}", err_str)));
    }

    let mut tables = Vec::new();

    loop {
        // SAFETY: stmt was just successfully prepared above and is not yet finalized.
        let step_ret = unsafe { ffi::sqlite3_step(stmt) };
        match step_ret {
            ffi::SQLITE_ROW => {
                // SAFETY: stmt is on a valid row; column index 0 is the only
                // selected column (name) in this query.
                unsafe {
                    let name =
                        std::ffi::CStr::from_ptr(ffi::sqlite3_column_text(stmt, 0) as *const i8)
                            .to_string_lossy()
                            .into_owned();
                    tables.push(name);
                }
            }
            ffi::SQLITE_DONE => break,
            _ => {
                let err = unsafe { ffi::sqlite3_errmsg(db_conn) };
                let err_str =
                    unsafe { std::ffi::CStr::from_ptr(err).to_string_lossy().into_owned() };
                unsafe { ffi::sqlite3_finalize(stmt) };
                return Err(JsValue::from_str(&format!("Step failed: {}", err_str)));
            }
        }
    }

    // SAFETY: stmt is non-null and was successfully prepared; finalize is safe
    // to call exactly once when done stepping.
    unsafe { ffi::sqlite3_finalize(stmt) };

    Ok(tables)
}
