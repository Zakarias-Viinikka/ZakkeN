mod black_magic;
mod black_magic_read;
mod create_sql_statements;
pub mod create_table;
pub mod public_data_shapes;
pub mod table_row;
use public_data_shapes::*;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct LiveForever {
    db_conn: Option<rusqlite::Connection>,
    conn_name: String,
    sahpool_util: Option<sqlite_wasm_vfs::sahpool::OpfsSAHPoolUtil>,
}

macro_rules! unwrap_or_bail {
    ($result:expr) => {
        match $result {
            Ok(v) => v,
            Err(e) => return e.serialize_wrapper(),
        }
    };
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
            conn_name,
        })
    }

    pub async fn create_table(&self, data: JsValue) -> Vec<u8> {
        let data = unwrap_or_bail!(CreateTableIn::cure_from_js_value(data));

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
        ok_serialized()
    }

    pub async fn close_conn_js(&mut self) -> Result<(), JsValue> {
        if let Some(conn) = self.db_conn.take() {
            if let Err(e) = black_magic::close_conn(conn) {
                return Err(JsValue::from(e));
            }
        }

        if let Some(util) = self.sahpool_util.take() {
            if let Err(e) = util.pause_vfs() {
                return Err(JsValue::from(DbError::ConnError(e.to_string())));
            }
        }

        Ok(())
    }

    pub async fn list_tables(&self) -> Vec<u8> /*Result<Vec<String>, JsValue>*/ {
        let conn = unwrap_or_bail!(self.conn());

        let list_of_table_names = unwrap_or_bail!(black_magic::list_tables(conn));

        ListTablesOut {
            table_names: list_of_table_names,
        }
        .serialize_wrapper()
    }

    pub async fn get_data(&self, data: Vec<u8>) -> Vec<u8> {
        let get_data_in = unwrap_or_bail!(GetDataIn::deserialize_wrapper(&data));

        let conn = unwrap_or_bail!(self.conn());

        let result = black_magic_read::read_from_db(conn, &get_data_in);

        match result {
            //let result: Vec<Vec<String>>
            Ok(result) => GetDataOut { rows: result }.serialize_wrapper(),
            Err(e) => e.serialize_wrapper(),
        }
    }

    pub async fn get_data_ordered(&self, data: Vec<u8>) -> Vec<u8> {
        let get_data_ordered_in = unwrap_or_bail!(GetDataOrderedIn::deserialize_wrapper(&data));

        let conn = unwrap_or_bail!(self.conn());

        let result = black_magic_read::read_from_db_ordered(conn, &get_data_ordered_in);

        match result {
            Ok(rows) => GetDataOut { rows }.serialize_wrapper(),
            Err(e) => e.serialize_wrapper(),
        }
    }

    pub async fn insert_data(&self, data: Vec<u8>) -> Vec<u8> {
        let input = unwrap_or_bail!(InsertDataIn::deserialize_wrapper(&data));

        let conn = unwrap_or_bail!(self.conn());

        match black_magic::insert_into_table(conn, &input.table_name, input.values) {
            Ok(()) => ok_serialized(),
            Err(e) => e.serialize_wrapper(),
        }
    }

    pub async fn drop_table(&self, data: Vec<u8>) -> Vec<u8> {
        let input = unwrap_or_bail!(DropTableIn::deserialize_wrapper(&data));

        let conn = match self.conn() {
            Ok(c) => c,
            Err(e) => return e.serialize_wrapper(),
        };

        match black_magic::drop_table(conn, &input.table_name) {
            Ok(()) => ok_serialized(),
            Err(e) => e.serialize_wrapper(),
        }
    }

    pub async fn edit_col_in_row(&self, data: Vec<u8>) -> Vec<u8> {
        let input = unwrap_or_bail!(EditColInRowIn::deserialize_wrapper(&data));
        let conn = unwrap_or_bail!(self.conn());

        unwrap_or_bail!(black_magic::edit_col_in_row(
            conn,
            &input.table_name,
            &input.row_id,
            &input.column,
            &input.new_value
        ));

        ok_serialized()
    }

    pub async fn check_table(&self, data: Vec<u8>) -> Vec<u8> {
        let input = unwrap_or_bail!(CheckTableIn::deserialize_wrapper(&data));
        let conn = unwrap_or_bail!(self.conn());

        let result = black_magic::table_shape(conn, &input.table_name);
        let columns = unwrap_or_bail!(result);

        CheckTableOut { columns: columns }.serialize_wrapper()
    }

    pub async fn delete_row(&self, data: Vec<u8>) -> Vec<u8> {
        let input = unwrap_or_bail!(DeleteRowIn::deserialize_wrapper(&data));
        let conn = unwrap_or_bail!(self.conn());

        unwrap_or_bail!(black_magic::delete_row(
            conn,
            &input.table_name,
            &input.row_id
        ));

        ok_serialized()
    }

    pub async fn swap_columns(&self, data: Vec<u8>) -> Vec<u8> {
        let input = unwrap_or_bail!(SwapColumnsIn::deserialize_wrapper(&data));
        let conn = unwrap_or_bail!(self.conn());

        let get_value = |row_id: &str| -> Result<table_row::Col, DbError> {
            let get_data_in = GetDataIn {
                table_name: input.table_name.clone(),
                arguments: vec![SelectArgument::XEqualY {
                    x: "id".to_string(),
                    y: row_id.to_string(),
                }],
                columns_to_read: vec![input.column.clone()],
            };

            let rows = black_magic_read::read_from_db(conn, &get_data_in)?;
            let row = rows
                .into_iter()
                .next()
                .ok_or_else(|| DbError::IllegalInput(format!("No row found for id {}", row_id)))?;

            let col = row.cols.into_iter().next().ok_or_else(|| {
                DbError::IllegalInput(format!("No column value for id {}", row_id))
            })?;

            Ok(col)
        };

        let value1 = unwrap_or_bail!(get_value(&input.row_id_1));
        let value2 = unwrap_or_bail!(get_value(&input.row_id_2));

        unwrap_or_bail!(black_magic::edit_col_in_row(
            conn,
            &input.table_name,
            &input.row_id_1,
            &input.column,
            &value2
        ));

        unwrap_or_bail!(black_magic::edit_col_in_row(
            conn,
            &input.table_name,
            &input.row_id_2,
            &input.column,
            &value1
        ));

        ok_serialized()
    }

    pub async fn create_index(&self, data: Vec<u8>) -> Vec<u8> {
        let input = unwrap_or_bail!(CreateIndexIn::deserialize_wrapper(&data));
        let conn = unwrap_or_bail!(self.conn());

        unwrap_or_bail!(black_magic::create_index(
            conn,
            &input.table_name,
            &input.column_name
        ));

        ok_serialized()
    }

    pub async fn check_index(&self, data: Vec<u8>) -> Vec<u8> {
        let input = unwrap_or_bail!(CheckIndexIn::deserialize_wrapper(&data));
        let conn = unwrap_or_bail!(self.conn());

        let is_indexed = unwrap_or_bail!(black_magic::check_index(
            conn,
            &input.table_name,
            &input.column_name
        ));

        CheckIndexOut { is_indexed }.serialize_wrapper()
    }

    pub async fn add_column(&self, data: Vec<u8>) -> Vec<u8> {
        let input = unwrap_or_bail!(AddColumnIn::deserialize_wrapper(&data));
        let conn = unwrap_or_bail!(self.conn());

        unwrap_or_bail!(black_magic::add_column(
            conn,
            &input.table_name,
            input.column
        ));

        ok_serialized()
    }

    pub async fn remove_column(&self, data: Vec<u8>) -> Vec<u8> {
        let input = unwrap_or_bail!(RemoveColumnIn::deserialize_wrapper(&data));
        let conn = unwrap_or_bail!(self.conn());

        unwrap_or_bail!(black_magic::remove_column(
            conn,
            &input.table_name,
            &input.column_name
        ));

        ok_serialized()
    }

    pub async fn export_database(&self, _data: Vec<u8>) -> Vec<u8> {
        let util = match self.sahpool_util.as_ref() {
            Some(u) => u,
            None => {
                return DbError::ConnError("SAH pool not connected".to_string())
                    .serialize_wrapper();
            }
        };

        let bytes = unwrap_or_bail!(black_magic::export_database(util, &self.conn_name));

        ExportDatabaseOut { data: bytes }.serialize_wrapper()
    }

    pub async fn export_tables(&self, data: Vec<u8>) -> Vec<u8> {
        let input = unwrap_or_bail!(ExportTablesIn::deserialize_wrapper(&data));
        let conn = unwrap_or_bail!(self.conn());

        let mut tables = Vec::new();

        for table_name in input.table_names {
            let columns = unwrap_or_bail!(black_magic::table_shape(conn, &table_name));

            let get_in = GetDataIn {
                table_name: table_name.clone(),
                arguments: vec![SelectArgument::All],
                columns_to_read: Vec::new(),
            };

            let rows = unwrap_or_bail!(black_magic_read::read_from_db(conn, &get_in));

            tables.push(TableExport {
                table_name,
                columns,
                rows,
            });
        }

        ExportTablesOut { tables }.serialize_wrapper()
    }

    pub async fn create_table_from_export(&self, data: Vec<u8>) -> Vec<u8> {
        let input = unwrap_or_bail!(CreateTableFromExportIn::deserialize_wrapper(&data));
        let conn = unwrap_or_bail!(self.conn());

        unwrap_or_bail!(black_magic::create_table_from_export(
            conn,
            &input.table_name,
            &input.table
        ));

        ok_serialized()
    }

    pub async fn copy_table(&self, data: Vec<u8>) -> Vec<u8> {
        let input = unwrap_or_bail!(CopyTableIn::deserialize_wrapper(&data));
        let conn = unwrap_or_bail!(self.conn());

        unwrap_or_bail!(black_magic::copy_table(
            conn,
            &input.source_table_name,
            &input.new_table_name
        ));

        ok_serialized()
    }

    fn conn(&self) -> Result<&rusqlite::Connection, DbError> {
        self.db_conn
            .as_ref()
            .ok_or_else(|| DbError::ConnError("Database not connected".to_string()))
    }
}
