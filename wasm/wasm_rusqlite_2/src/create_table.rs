#![warn(unused)]
#![allow(dead_code)]

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct ColumnDef(
    pub String, // name
    pub String, // column type
    pub bool,   // primary key
    pub bool,   // not null
    pub bool,   // unique
    pub String, // default value
    pub bool,   // autoincrement
);

pub struct ColumnDefBuilder(
    pub String,     // name
    pub ColumnType, // column type
    pub bool,       // primary key
    pub bool,       // not null
    pub bool,       // unique
    pub String,     // default value
    pub bool,       // autoincrement
);

pub fn id_column() -> ColumnDef {
    builder_to_column_def(ColumnDefBuilder(
        "id".to_string(),
        ColumnType::Integer,
        true,
        true,
        true,
        "".to_string(),
        true,
    ))
}

pub fn default_col(column_type: ColumnType) -> ColumnDef {
    builder_to_column_def(ColumnDefBuilder(
        "".to_string(),
        column_type,
        false,
        false,
        false,
        "".to_string(),
        false,
    ))
}

pub fn col_with_default_value(column_type: ColumnType, default_value: String) -> ColumnDef {
    builder_to_column_def(ColumnDefBuilder(
        "".to_string(),
        column_type,
        false,
        false,
        false,
        default_value,
        false,
    ))
}

pub enum ColumnType {
    Integer,
    Text,
    Real,
    Blob,
}

pub fn builder_to_column_def(builder: ColumnDefBuilder) -> ColumnDef {
    let column_type = match builder.1 {
        ColumnType::Integer => "INTEGER".to_string(),
        ColumnType::Text => "TEXT".to_string(),
        ColumnType::Real => "REAL".to_string(),
        ColumnType::Blob => "BLOB".to_string(),
    };
    ColumnDef(
        builder.0,
        column_type,
        builder.2,
        builder.3,
        builder.4,
        builder.5,
        builder.6,
    )
}
