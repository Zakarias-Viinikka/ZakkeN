mod black_magic;
mod black_magic_read;
mod create_sql_statements;
mod create_table;
pub mod public_data_shapes;
pub mod table_row;
mod utils;
use public_data_shapes::*;

use create_table::ColumnDef;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct LiveForever {
    db_conn: Option<rusqlite::Connection>,
    sahpool_util: Option<sqlite_wasm_vfs::sahpool::OpfsSAHPoolUtil>,
}

#[wasm_bindgen]
impl LiveForever {
    pub async fn new(conn_name: String) -> Result<LiveForever, JsValue> {
        let (sahpool_util, db_conn) = black_magic::create_local_db_connection(&conn_name)
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(LiveForever {
            db_conn: Some(db_conn),
            sahpool_util: Some(sahpool_util),
        })
    }

    pub async fn create_table(&self, data: JsValue) -> Vec<u8> {
        let data = match CreateTableIn::cure_from_js_value(data) {
            Ok(data) => data,
            Err(e) => return e.serialize_wrapper(),
        };

        let (table_name, columns) = (data.table_name, data.columns);

        let Some(conn) = self.db_conn.as_ref() else {
            return DbError::ConnError("Database not connected".to_string()).serialize_wrapper();
        };

        let result = black_magic::create_table(conn, &table_name, columns);
        result.serialize_wrapper()
    }

    pub async fn close_conn(&mut self) -> Vec<u8> {
        if let Some(conn) = self.db_conn.take() {
            if let Err(e) = black_magic::close_conn(conn) {
                return e.serialize_wrapper();
            }
        }

        if let Some(util) = self.sahpool_util.take() {
            if let Err(e) = util.pause_vfs() {
                return DbError::ConnError(e.to_string()).serialize_wrapper();
            }
        }
        public_data_shapes::ok_serialized()
    }

    pub async fn list_tables(&self) -> Vec<u8> /*Result<Vec<String>, JsValue>*/ {
        let conn = match self.conn() {
            Ok(c) => c,
            Err(e) => return e.serialize_wrapper(),
        };

        let list_of_table_names = match black_magic::list_tables(conn) {
            Ok(l) => l,
            Err(e) => return e.serialize_wrapper(),
        };

        public_data_shapes::ListTablesOut {
            table_names: list_of_table_names,
        }
        .serialize_wrapper()
    }

    pub async fn get_data(
        &self,
        table_name: String,
        arguments: String,
        columns_to_read: Vec<String>,
    ) -> Vec<u8> {
        let conn = self.conn()?;
        let result = black_magic_read::read_from_db(
            conn,
            table_name,            // String → impl AsRef<str>
            &[arguments.as_str()], // single condition as a slice of &str
            &columns_to_read,      // &Vec<String> → &[impl AsRef<str>]
        )
        .map_err(|e| JsValue::from(e.to_string()))?;
        let result =
            serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from(e.to_string()))?;
        Ok(result) //serde-wasm-bindgen = "0.6.5"
    }

    pub async fn get_data_ordered(
        &self,
        table_name: String,
        arguments: String,
        columns_to_read: Vec<String>,
        order_by: String,
    ) -> Result<JsValue, JsValue> {
        let conn = self.conn()?;
        let result = black_magic_read::read_from_db_ordered(
            conn,
            table_name,
            &[arguments.as_str()],
            &columns_to_read,
            &order_by,
        )
        .map_err(|e| JsValue::from(e.to_string()))?;
        let result =
            serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from(e.to_string()))?;
        Ok(result)
    }

    pub async fn insert_data(
        &self,
        table_name: String,
        col_names: Vec<String>, // array of column names from JS
        vals: Vec<String>,      // array of values from JS
    ) -> Result<(), JsValue> {
        let conn = self.conn()?;
        // zip the two arrays into (column, value) pairs
        let values: Vec<(String, String)> = col_names.into_iter().zip(vals.into_iter()).collect();

        black_magic::insert_into_table(conn, &table_name, values)
            .map_err(|e| JsValue::from(e.to_string()))?;
        Ok(())
    }

    pub async fn drop_table(&self, table_name: String) -> Result<(), JsValue> {
        let conn = self.conn()?;
        black_magic::drop_table(conn, &table_name)?;
        Ok(())
    }

    pub async fn edit_col_in_row(
        &self,
        table_name: String,
        row_id: String,
        column: String,
        new_value: String,
    ) -> Result<(), JsValue> {
        let conn = self.conn()?;
        black_magic::edit_col_in_row(conn, &table_name, &row_id, (column, new_value))
            .map_err(|e| JsValue::from(e.to_string()))?;
        Ok(())
    }

    pub async fn check_table(&self, table_name: &str) -> Result<Vec<String>, JsValue> {
        let conn = self.conn()?;
        black_magic::table_shape(conn, table_name)
    }

    pub async fn delete_row(&self, table_name: String, row_id: String) -> Result<(), JsValue> {
        let conn = self.conn()?;
        black_magic::delete_row(conn, &table_name, &row_id)
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
        let value1 = self
            .get_data(
                table_name.clone(),
                format!("id = {}", row_id_1),
                vec![column.clone()],
            )
            .await?;

        let value2 = self
            .get_data(
                table_name.clone(),
                format!("id = {}", row_id_2),
                vec![column.clone()],
            )
            .await?;

        let value1: Vec<Vec<String>> =
            serde_wasm_bindgen::from_value(value1).map_err(|e| JsValue::from(e.to_string()))?;

        let value2: Vec<Vec<String>> =
            serde_wasm_bindgen::from_value(value2).map_err(|e| JsValue::from(e.to_string()))?;

        let value1 = value1
            .into_iter()
            .next()
            .and_then(|mut row| row.pop())
            .ok_or_else(|| JsValue::from_str("No row found for first id"))?;

        let value2 = value2
            .into_iter()
            .next()
            .and_then(|mut row| row.pop())
            .ok_or_else(|| JsValue::from_str("No row found for second id"))?;

        self.edit_col_in_row(table_name.clone(), row_id_1, column.clone(), value2)
            .await?;

        self.edit_col_in_row(table_name, row_id_2, column, value1)
            .await?;

        Ok(())
    }

    pub async fn delete_table(&self, table_name: String) -> Result<(), JsValue> {
        let conn = self.conn()?;
        black_magic::drop_table(conn, &table_name)?; // already exists
        Ok(())
    }

    pub async fn create_index(
        &self,
        table_name: String,
        column_name: String,
    ) -> Result<(), JsValue> {
        let conn = self.conn()?;
        black_magic::create_index(conn, &table_name, &column_name) // PLACEHOLDER - doesn't exist yet
            .map_err(|e| JsValue::from(e.to_string()))?;
        Ok(())
    }

    fn conn(&self) -> Result<&rusqlite::Connection, DbError> {
        self.db_conn
            .as_ref()
            .ok_or_else(|| DbError::ConnError("Database not connected".to_string()))
    }
}
