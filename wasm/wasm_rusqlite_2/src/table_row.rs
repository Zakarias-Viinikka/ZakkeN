use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
pub enum Col {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}
