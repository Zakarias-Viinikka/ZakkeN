//use js_sys::{Array, Uint8Array};
// new for sahpool
use sqlite_wasm_rs::{self as ffi};
//old //use sqlite_wasm_rs as ffi; //necessary as far as i can tell.

use crate::create_sql_statements::*;
use crate::db_table::*;

use anyhow::{Result, anyhow, bail};
use std::ffi::{CStr, CString}; //let sql_cstr = CString::new(sql).map_err(|e| anyhow!("CString conversion failed: {}", e))?;

//SELECT col1, col2 FROM table_name WHERE id = 1;
pub fn read_from_db(
    db: *mut ffi::sqlite3,
    table_name: impl AsRef<str>,
    arguments: &[impl AsRef<str>],
    columns_to_read: &[impl AsRef<str>],
) -> Result<Vec<Vec<String>>> {
    if db.is_null() {
        bail!("db pointer is null");
    }

    let sql = generate_read_from_table_sql(&table_name, arguments, columns_to_read);

    let sql_cstr = CString::new(sql)?;
    let mut stmt: *mut ffi::sqlite3_stmt = std::ptr::null_mut();

    //ret is sqlite3_prepare_v2 threw an error or not.
    // this block of code changes stmt from being empty to being
    // the representation of the sql statement the db wants
    let ret = unsafe {
        ffi::sqlite3_prepare_v2(db, sql_cstr.as_ptr(), -1, &mut stmt, std::ptr::null_mut())
    };
    if ret != ffi::SQLITE_OK {
        bail!("prepare failed: {}", ffi::code_to_str(ret),);
    }

    let mut rows = Vec::new();
    loop {
        //basically the equivalent of iter.next.
        // step_ret is just the status code for
        // "if .next returned something or not".
        let step_ret = unsafe { ffi::sqlite3_step(stmt) };
        if step_ret == ffi::SQLITE_DONE {
            break;
        } else if step_ret == ffi::SQLITE_ROW {
            //this just finds out how many cols are in the row
            // table.columns.len
            let col_count = unsafe { ffi::sqlite3_column_count(stmt) };
            //the array size is known once col_count is known, but not at
            // compile time. so this is the "efficient way" to do the list
            let mut row = Vec::with_capacity(col_count as usize);
            //sqlite3_column_text returns a pointer to
            // the bytes of the text value in column i of the current row.
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

unsafe fn c_str_to_string(ptr: *const std::os::raw::c_char) -> String {
    if ptr.is_null() {
        String::new()
    } else {
        CStr::from_ptr(ptr).to_string_lossy().into_owned()
    }
}
