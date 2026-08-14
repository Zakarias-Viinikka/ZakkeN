//use serde_wasm_bindgen::to_value;
#[derive(serde::Deserialize)]
pub struct ColumnDef(
    pub String, // name
    pub String, // column type
    pub bool,   // primary key
    pub bool,   // not null
    pub bool,   // unique
    pub String, // default value
    pub bool,   // autoincrement
);
