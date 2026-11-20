mod black_magic;
mod black_magic_read;
mod create_sql_statements;
mod create_table_col_def;
mod utils;

use create_table_col_def::ColumnDef;

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

    pub async fn create_table(&self, table_name: String, columns: JsValue) -> Result<(), JsValue> {
        let columns: Vec<ColumnDef> =
            serde_wasm_bindgen::from_value(columns).map_err(|e| JsValue::from(e.to_string()))?;

        let conn = self
            .db_conn
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Database not connected"))?;

        black_magic::create_table(conn, &table_name, columns)
            .map_err(|e| JsValue::from(e.to_string()))?;

        Ok(())
    }

    pub async fn close_conn(&mut self) -> Result<(), JsValue> {
        if let Some(conn) = self.db_conn.take() {
            black_magic::close_conn(conn).map_err(|e| JsValue::from(e.to_string()))?;
        }

        if let Some(util) = self.sahpool_util.take() {
            util.pause_vfs().map_err(|e| JsValue::from(e.to_string()))?;
        }

        Ok(())
    }

    pub async fn list_tables(&self) -> Result<Vec<String>, JsValue> {
        let conn = self.conn()?;
        black_magic::list_tables(conn)
    }

    pub async fn get_data(
        &self,
        table_name: String,
        arguments: String,
        columns_to_read: Vec<String>,
    ) -> Result<JsValue, JsValue> {
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

    fn conn(&self) -> Result<&rusqlite::Connection, JsValue> {
        self.db_conn
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Database not connected"))
    }
}
