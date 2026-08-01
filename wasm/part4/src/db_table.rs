use serde::{Deserialize, Serialize};
#[derive(Clone, Serialize, Deserialize)]
pub enum ColumnType {
    Integer,
    Text,
    Real, // for floating-point numbers
    Blob, // for binary data
}

impl ColumnType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ColumnType::Integer => "INTEGER",
            ColumnType::Text => "TEXT",
            ColumnType::Real => "REAL",
            ColumnType::Blob => "BLOB",
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Column {
    pub column_name: String,     // now public
    pub column_type: ColumnType, // now public
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Table {
    pub table_name: String,   // now public
    pub columns: Vec<Column>, // now public
}
