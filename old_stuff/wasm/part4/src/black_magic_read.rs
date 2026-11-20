//use js_sys::{Array, Uint8Array};
// new for sahpool
use sqlite_wasm_rs::{self as ffi};
//old //use sqlite_wasm_rs as ffi; //necessary as far as i can tell.

use crate::create_sql_statements::*;

use anyhow::{Result, bail};
use std::ffi::{CStr, CString}; //let sql_cstr = CString::new(sql).map_err(|e| anyhow!("CString conversion failed: {}", e))?;

fn black_magic_read_from(db: *mut ffi::sqlite3, sql: String) -> Result<Vec<Vec<String>>> {
    if db.is_null() {
        bail!("db pointer is null");
    }

    let sql_cstr = CString::new(sql)?;
    let mut stmt: *mut ffi::sqlite3_stmt = std::ptr::null_mut();

    let ret = unsafe {
        ffi::sqlite3_prepare_v2(db, sql_cstr.as_ptr(), -1, &mut stmt, std::ptr::null_mut())
    };
    if ret != ffi::SQLITE_OK {
        bail!("prepare failed: {}", ffi::code_to_str(ret));
    }

    let mut rows = Vec::new();
    loop {
        let step_ret = unsafe { ffi::sqlite3_step(stmt) };
        if step_ret == ffi::SQLITE_DONE {
            break;
        } else if step_ret == ffi::SQLITE_ROW {
            let col_count = unsafe { ffi::sqlite3_column_count(stmt) };
            let mut row = Vec::with_capacity(col_count as usize);
            for i in 0..col_count {
                let text_ptr = unsafe { ffi::sqlite3_column_text(stmt, i) };
                row.push(unsafe { c_str_to_string(text_ptr as *const std::os::raw::c_char) });
            }
            rows.push(row);
        } else {
            let err_msg = unsafe { c_str_to_string(ffi::sqlite3_errmsg(db)) };
            unsafe {
                ffi::sqlite3_finalize(stmt);
            }
            bail!("sqlite3_step error: {}", err_msg);
        }
    }

    let fin_ret = unsafe { ffi::sqlite3_finalize(stmt) };
    if fin_ret != ffi::SQLITE_OK {
        bail!("finalize failed: {}", ffi::code_to_str(fin_ret));
    }
    Ok(rows)
}

pub fn read_from_db(
    db: *mut ffi::sqlite3,
    table_name: impl AsRef<str>,
    arguments: &[impl AsRef<str>],
    columns_to_read: &[impl AsRef<str>],
) -> Result<Vec<Vec<String>>> {
    let sql = generate_read_from_table_sql(&table_name, arguments, columns_to_read);
    black_magic_read_from(db, sql)
}

pub fn read_from_db_ordered(
    db: *mut ffi::sqlite3,
    table_name: impl AsRef<str>,
    arguments: &[impl AsRef<str>],
    columns_to_read: &[impl AsRef<str>],
    order_by: &str,
) -> Result<Vec<Vec<String>>> {
    let sql = generate_get_data_by_order_sql(&table_name, arguments, columns_to_read, order_by);
    black_magic_read_from(db, sql)
}

unsafe fn c_str_to_string(ptr: *const std::os::raw::c_char) -> String {
    if ptr.is_null() {
        String::new()
    } else {
        CStr::from_ptr(ptr).to_string_lossy().into_owned()
    }
}
