use crate::{LiveForever, create_table};
use anyhow::{Result, anyhow};
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

#[derive(Serialize, Deserialize, Debug)]
pub struct ListTablesOut {
    pub table_names: Vec<String>,
}
/*Result<Vec<String>, JsValue>*/

pub trait Convert: Serialize + DeserializeOwned {
    fn serialize_wrapper(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap_or_else(|e| {
            bincode::serialize(&DbError::SerializeError(e.to_string())).unwrap_or_else(|_| {
                console::error_1(&JsValue::from("Serialization had an error, but trying to serialize the error message also failed. Returning empty — deserializing will fail, swallowing the actual error."));
                Vec::new()
            })
        })
    }

    fn deserialize_wrapper(data: &[u8]) -> Result<Self> {
        bincode::deserialize(data).map_err(|e| anyhow!(e))
    }

    fn cure_from_js_value(value: JsValue) -> Result<Self> {
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
