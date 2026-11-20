// TODO: SQL injection - generate_delete_sql's `id` and generate_get_data_by_order_sql's
// `order_by` are spliced in raw, unquoted/unvalidated.

use crate::create_table;
use crate::table_row;
use anyhow::Result;
use js_sys::Uint8Array;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;
use web_sys::console;

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateTableIn {
    pub table_name: String,
    pub columns: Vec<create_table::ColumnDef>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateTableOut {
    pub result: Result<(), DbError>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ListTablesOut {
    pub table_names: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetDataIn {
    pub table_name: String,
    pub arguments: Vec<SelectArgument>,
    pub columns_to_read: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SelectArgument {
    XEqualY { x: String, y: String },
    XNotEqualY { x: String, y: String },
    XGreaterThanY { x: String, y: String },
    XLessThanY { x: String, y: String },
    XGreaterThanOrEqualY { x: String, y: String },
    XLessThanOrEqualY { x: String, y: String },
    XLikeY { x: String, y: String },
    XInY { x: String, y: Vec<String> },
    All,
}

impl SelectArgument {
    pub fn to_sql_condition(&self) -> String {
        fn quote_value(value: &str) -> String {
            format!("'{}'", value.replace('\'', "''"))
        }
        fn quote_ident(ident: &str) -> String {
            format!("\"{}\"", ident.replace('"', "\"\""))
        }

        match self {
            SelectArgument::XEqualY { x, y } => format!("{} = {}", quote_ident(x), quote_value(y)),
            SelectArgument::XNotEqualY { x, y } => {
                format!("{} != {}", quote_ident(x), quote_value(y))
            }
            SelectArgument::XGreaterThanY { x, y } => {
                format!("{} > {}", quote_ident(x), quote_value(y))
            }
            SelectArgument::XLessThanY { x, y } => {
                format!("{} < {}", quote_ident(x), quote_value(y))
            }
            SelectArgument::XGreaterThanOrEqualY { x, y } => {
                format!("{} >= {}", quote_ident(x), quote_value(y))
            }
            SelectArgument::XLessThanOrEqualY { x, y } => {
                format!("{} <= {}", quote_ident(x), quote_value(y))
            }
            SelectArgument::XLikeY { x, y } => {
                format!("{} LIKE {}", quote_ident(x), quote_value(y))
            }
            SelectArgument::XInY { x, y } => {
                let quoted_values: Vec<String> = y.iter().map(|v| quote_value(v)).collect();
                format!("{} IN ({})", quote_ident(x), quoted_values.join(", "))
            }
            SelectArgument::All => String::new(),
        }
    }
}

//let result: Vec<Vec<String>>
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetDataOut {
    pub rows: Vec<table_row::Row>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetDataOrderedIn {
    pub table_name: String,
    pub arguments: Vec<SelectArgument>,
    pub columns_to_read: Vec<String>,
    pub order_by: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ColumnValue {
    pub column_name: String,
    pub value: table_row::Col,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InsertDataIn {
    pub table_name: String,
    pub values: Vec<ColumnValue>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InsertDataOut {
    pub result: Result<(), DbError>,
}

// public_data_shapes.rs
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DropTableIn {
    pub table_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EditColInRowIn {
    pub table_name: String,
    pub row_id: String,
    pub column: String,
    pub new_value: table_row::Col,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CheckTableIn {
    pub table_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TableColumnInfo {
    pub cid: i64,
    pub name: String,
    pub type_name: String,
    pub not_null: bool,
    pub default_value: Option<String>,
    pub primary_key: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CheckTableOut {
    pub columns: Vec<TableColumnInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeleteRowIn {
    pub table_name: String,
    pub row_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SwapColumnsIn {
    pub table_name: String,
    pub row_id_1: String,
    pub row_id_2: String,
    pub column: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateIndexIn {
    pub table_name: String,
    pub column_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CheckIndexIn {
    pub table_name: String,
    pub column_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CheckIndexOut {
    pub is_indexed: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AddColumnIn {
    pub table_name: String,
    pub column: create_table::ColumnDef,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExportDatabaseIn {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExportDatabaseOut {
    pub data: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RemoveColumnIn {
    pub table_name: String,
    pub column_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExportTablesIn {
    pub table_names: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TableExport {
    pub table_name: String,
    pub columns: Vec<TableColumnInfo>,
    pub rows: Vec<table_row::Row>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExportTablesOut {
    pub tables: Vec<TableExport>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateTableFromExportIn {
    pub table_name: String,
    pub table: TableExport,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CopyTableIn {
    pub source_table_name: String,
    pub new_table_name: String,
}

pub trait Convert: Serialize + DeserializeOwned {
    fn serialize_wrapper(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap_or_else(|e| {
            bincode::serialize(&DbError::SerializeError(e.to_string())).unwrap_or_else(|_| {
                console::error_1(&JsValue::from("Serialization had an error, but trying to serialize the error message also failed. Returning empty — deserializing will fail, swallowing the actual error."));
                Vec::new()
            })
        })
    }

    fn deserialize_wrapper(data: &[u8]) -> Result<Self, DbError> {
        bincode::deserialize(data).map_err(|e| DbError::CureFail(e.to_string()))
    }

    fn cure_from_js_value(value: JsValue) -> Result<Self, DbError> {
        let bytes = Uint8Array::from(value).to_vec();
        <Self>::deserialize_wrapper(&bytes)
    }
}

pub fn ok_serialized() -> Vec<u8> {
    Ok::<(), DbError>(()).serialize_wrapper()
}

impl<T: Serialize + DeserializeOwned> Convert for T {}

#[derive(Serialize, Deserialize, Debug)]
pub enum DbError {
    CureFail(String),
    ConnError(String),
    IllegalInput(String),
    SqlExecuteFail(String),
    SerializeError(String),
    BadCode(String), //this is only meant to error if there is something wrong with the code
}

impl From<DbError> for JsValue {
    fn from(e: DbError) -> JsValue {
        JsValue::from_str(&format!("{:?}", e))
    }
}
