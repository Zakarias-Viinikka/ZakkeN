use protocol::new_table::*;

pub const COLUMN_NAME: &str = "happy_col_name";

pub fn column_definitions() -> Vec<ColumnDef> {
    vec![id_column(), default_col(ColumnType::Text, COLUMN_NAME)]
}
