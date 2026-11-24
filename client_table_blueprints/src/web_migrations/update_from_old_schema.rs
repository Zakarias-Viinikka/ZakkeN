use protocol::error::DbError;

use crate::web_migrations::schema_versions::SchemaVersion;
use crate::web_migrations::schemas::{version1, version2};

pub async fn update_until_newest_version(
    mut current: SchemaVersion,
) -> Result<SchemaVersion, DbError> {
    loop {
        let next = update_once(current).await?;
        if next == current {
            return Ok(next);
        }
        current = next;
    }
}

pub async fn update_once(current: SchemaVersion) -> Result<SchemaVersion, DbError> {
    match current {
        SchemaVersion::Version0 => {
            version1::update_from_old_version().await?;
            Ok(SchemaVersion::Version1)
        }
        SchemaVersion::Version1 => {
            version2::update_from_old_version().await?;
            Ok(SchemaVersion::Version2)
        }
        SchemaVersion::Version2 => Ok(SchemaVersion::Version2),
    }
}