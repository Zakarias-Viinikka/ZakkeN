use protocol::error::DbError;

use crate::web_migrations::helpers::ADbTable;
use crate::web_migrations::schema_versions::{CURRENT_VERSION, SchemaVersion};
use crate::web_migrations::schemas::{version0, version1, version2};

pub fn entire_table() -> Vec<ADbTable> {
    match CURRENT_VERSION {
        SchemaVersion::Version0 => version0::entire_table(),
        SchemaVersion::Version1 => version1::entire_table(),
        SchemaVersion::Version2 => version2::entire_table(),
    }
}

pub async fn create_fts_table() -> Result<(), DbError> {
    match CURRENT_VERSION {
        SchemaVersion::Version0 => Ok(()),
        SchemaVersion::Version1 => version1::create_fts_table().await,
        SchemaVersion::Version2 => version2::create_fts_table().await,
    }
}