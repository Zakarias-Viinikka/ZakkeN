use crate::schema::migration::helpers::ADbTable;
use crate::schema::migration::schema_versions::{CURRENT_VERSION, SchemaVersion};
use crate::schema::migration::schemas::version0;

pub fn entire_table() -> Vec<ADbTable> {
    match CURRENT_VERSION {
        SchemaVersion::Version0 => version0::entire_table(),
    }
}
