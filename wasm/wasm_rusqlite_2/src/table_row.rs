use crate::DbError;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Row {
    pub cols: Vec<Col>,
}

impl Col {
    pub fn as_str(&self) -> Result<&str, String> {
        match self {
            Col::Text(s) => Ok(s),
            _ => Err("failed to convert to string".to_string()),
        }
    }

    pub fn as_int(&self) -> Result<&i64, String> {
        match self {
            Col::Integer(i) => Ok(i),
            _ => Err("failed to convert to integer".to_string()),
        }
    }

    pub fn as_real(&self) -> Result<&f64, String> {
        match self {
            Col::Real(r) => Ok(r),
            _ => Err("failed to convert to real".to_string()),
        }
    }

    pub fn as_blob(&self) -> Result<&[u8], String> {
        match self {
            Col::Blob(b) => Ok(b),
            _ => Err("failed to convert to blob".to_string()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Col {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}

pub fn col_to_string(col: &Col) -> Result<String, DbError> {
    match col {
        Col::Null => Err(DbError::IllegalInput("Cannot swap NULL value".to_string())),
        Col::Integer(i) => Ok(i.to_string()),
        Col::Real(f) => Ok(f.to_string()),
        Col::Text(s) => Ok(s.clone()),
        Col::Blob(_) => Err(DbError::IllegalInput("Cannot swap blob value".to_string())),
    }
}
