use protocol::error::DbError;

use crate::schema::migration::schema_versions::SchemaVersion;

pub async fn update_once(current: SchemaVersion) -> Result<SchemaVersion, DbError> {
    match current {
        SchemaVersion::Version0 => Ok(SchemaVersion::Version0),
    }
}

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
