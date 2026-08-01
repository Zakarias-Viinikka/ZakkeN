mod black_magic;
mod black_magic_read;
mod create_sql_statements;
mod create_table_col_def;
mod db_table;
mod utils;

use create_table_col_def::ColumnDef;

//use serde_wasm_bindgen::to_value;
use sqlite_wasm_rs::{self as ffi};
use wasm_bindgen::prelude::*;

use crate::db_table::*;
#[wasm_bindgen]
pub struct LiveForever {
    db_conn: *mut ffi::sqlite3,
}

#[wasm_bindgen]
impl LiveForever {
    pub async fn new(conn_name: String) -> Result<LiveForever, JsValue> {
        let db_conn = black_magic::create_local_db_connection(&conn_name)
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(LiveForever { db_conn: db_conn })
    }

    /*pub fn read_from_db(
        db: *mut ffi::sqlite3,
        table_name: String,
        identifier: String,
    ) */
    pub async fn get_data(
        &self,
        table_name: String,
        arguments: String,
        columns_to_read: Vec<String>,
    ) -> Result<JsValue, JsValue> {
        let result = black_magic_read::read_from_db(
            self.db_conn,
            table_name,            // String → impl AsRef<str>
            &[arguments.as_str()], // single condition as a slice of &str
            &columns_to_read,      // &Vec<String> → &[impl AsRef<str>]
        )
        .map_err(|e| JsValue::from(e.to_string()))?;
        let result =
            serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from(e.to_string()))?;
        Ok(result) //serde-wasm-bindgen = "0.6.5"
    }

    //console.log(state.change_data("new data"));
    pub async fn insert_data(
        &self,
        table_name: String,
        col_names: Vec<String>, // array of column names from JS
        vals: Vec<String>,      // array of values from JS
    ) -> Result<(), JsValue> {
        // zip the two arrays into (column, value) pairs
        let values: Vec<(String, String)> = col_names.into_iter().zip(vals.into_iter()).collect();

        black_magic::insert_into_table(self.db_conn, &table_name, values)
            .map_err(|e| JsValue::from(e.to_string()))?;
        Ok(())
    }

    pub async fn drop_table(&self, table_name: String) -> Result<(), JsValue> {
        black_magic::drop_table(self.db_conn, &table_name)?;
        Ok(())
    }

    //pub fn table_shape(db_conn: *mut ffi::sqlite3, table_name: &str) -> Result<Vec<String>, JsValue> {
    pub async fn check_table(&self, table_name: &str) -> Result<Vec<String>, JsValue> {
        black_magic::table_shape(self.db_conn, table_name)
    }
    pub async fn edit_col_in_row(
        &self,
        table_name: String,
        row_id: String,
        column: String,
        new_value: String,
    ) -> Result<(), JsValue> {
        black_magic::edit_col_in_row(self.db_conn, &table_name, &row_id, (column, new_value))
            .map_err(|e| JsValue::from(e.to_string()))?;
        Ok(())
    }

    pub async fn delete_row(&self, table_name: String, row_id: String) -> Result<(), JsValue> {
        black_magic::delete_row(self.db_conn, &table_name, &row_id)
            .map_err(|e| JsValue::from(e.to_string()))?;
        Ok(())
    }

    pub async fn swap_columns(
        &self,
        table_name: String,
        row_id_1: String,
        row_id_2: String,
        column: String,
    ) -> Result<(), JsValue> {
        let condition = format!("id = {}", row_id_1);
        let value1 = black_magic_read::read_from_db(
            self.db_conn,
            &table_name,           // &String works as impl AsRef<str>
            &[condition.as_str()], // single &str condition
            &[column.as_str()],    // single &str column
        )
        .map_err(|e| JsValue::from(e.to_string()))?;
        let value1 = value1
            .into_iter()
            .next()
            .ok_or_else(|| JsValue::from("No rows returned from database"))?
            .into_iter()
            .next()
            .ok_or_else(|| JsValue::from("Row has no columns"))?;

        let condition2 = format!("id = {}", row_id_2);
        let value2 = black_magic_read::read_from_db(
            self.db_conn,
            &table_name,
            &[condition2.as_str()], // row_id_2 is a String; .as_str() gives &str
            &[column.as_str()],
        )
        .map_err(|e| JsValue::from(e.to_string()))?;
        let value2 = value2
            .into_iter()
            .next()
            .ok_or_else(|| JsValue::from("No rows returned from database"))?
            .into_iter()
            .next()
            .ok_or_else(|| JsValue::from("Row has no columns"))?;

        black_magic::edit_col_in_row(self.db_conn, &table_name, &row_id_1, (&column, value2))
            .map_err(|e| JsValue::from(e.to_string()))?;
        black_magic::edit_col_in_row(self.db_conn, &table_name, &row_id_2, (&column, value1))
            .map_err(|e| JsValue::from(e.to_string()))?;
        Ok(())
    }

    pub async fn create_table(&self, table_name: String, columns: JsValue) -> Result<(), JsValue> {
        let columns: Vec<ColumnDef> =
            serde_wasm_bindgen::from_value(columns).map_err(|e| JsValue::from(e.to_string()))?;
        black_magic::create_table(self.db_conn, &table_name, columns) // PLACEHOLDER - doesn't exist yet
            .map_err(|e| JsValue::from(e.to_string()))?;
        Ok(())
    }

    pub async fn delete_table(&self, table_name: String) -> Result<(), JsValue> {
        black_magic::drop_table(self.db_conn, &table_name)?; // already exists
        Ok(())
    }

    pub async fn create_index(
        &self,
        table_name: String,
        column_name: String,
    ) -> Result<(), JsValue> {
        black_magic::create_index(self.db_conn, &table_name, &column_name) // PLACEHOLDER - doesn't exist yet
            .map_err(|e| JsValue::from(e.to_string()))?;
        Ok(())
    }

    pub async fn list_tables(&self) -> Result<Vec<String>, JsValue> {
        black_magic::list_tables(self.db_conn)
    }
}

// https://wasm-bindgen.github.io/wasm-bindgen/examples/wasm-in-web-worker.html
