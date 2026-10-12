use crate::schema::migration::helpers::ADbTable;
use crate::schema::table;

pub fn entire_table() -> Vec<ADbTable> {
    table::all_tables()
}
