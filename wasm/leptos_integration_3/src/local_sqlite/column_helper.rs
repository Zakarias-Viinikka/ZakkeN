use crate::local_sqlite::local_sqlite_wrapper::CreateTableColumnDef;
use serde::{Deserialize, Serialize};

/*
pub struct CreateTableColumnDef {
    pub name: String,
    pub col_type: String,
    pub primary_key: bool,
    pub not_null: bool,
    pub unique: bool,
    pub default_value: String,
    pub autoincrement: bool,
}
*/
// Convenience: an auto-incrementing integer primary key (the most common ID column).
pub fn id_column() -> CreateTableColumnDef {
    CreateTableColumnDef {
        name: "id".to_string(),
        col_type: "INTEGER".to_string(),
        primary_key: true,
        not_null: false,
        unique: false,
        default_value: String::new(),
        autoincrement: true,
    }
}

/// Simplest column: just a name and type, no constraints, no default.
pub fn column(name: &str, col_type: &str) -> CreateTableColumnDef {
    CreateTableColumnDef {
        name: name.to_string(),
        col_type: col_type.to_string(),
        primary_key: false,
        not_null: false,
        unique: false,
        default_value: String::new(),
        autoincrement: false,
    }
}

/// A boolean column (stored as INTEGER in SQLite) with a default value.
pub fn bool_column(name: &str, default_true: bool) -> CreateTableColumnDef {
    CreateTableColumnDef {
        name: name.to_string(),
        col_type: "TEXT".to_string(), // was "INTEGER"
        primary_key: false,
        not_null: false,
        unique: false,
        default_value: if default_true {
            "'true'".to_string() // was "1"
        } else {
            "'false'".to_string() // was "0"
        },
        autoincrement: false,
    }
}
