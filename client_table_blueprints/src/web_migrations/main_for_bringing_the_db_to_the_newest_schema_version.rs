use protocol::error::DbError;
use web_internal_db::db_helper;

use crate::tbl_every_block_in_existence;
use crate::web_migrations::db_actions_for_schema_version_table::{
    ensure_schema_version_table_exists, read_stored_schema_version, write_schema_version,
};
use crate::web_migrations::helpers::create_table_from_scratch;
use crate::web_migrations::schema_versions::{CURRENT_VERSION, SchemaVersion};
use crate::web_migrations::schemas::current;
use crate::web_migrations::update_from_old_schema::update_until_newest_version;

pub async fn bring_the_db_to_the_newest_schema_version() -> Result<(), DbError> {
    ensure_schema_version_table_exists().await?;

    let table_names = db_helper::list_tables().await?.table_names;
    let db_is_fresh = !table_names
        .iter()
        .any(|name| name.as_str() == tbl_every_block_in_existence::TABLE_NAME);

    if db_is_fresh {
        create_table_from_scratch(current::entire_table()).await?;
        current::create_fts_table().await?;
        write_schema_version(CURRENT_VERSION.to_int()).await?;
        return Ok(());
    }

    let stored_version = read_stored_schema_version().await?;
    let version_the_db_is_on = match stored_version {
        None => SchemaVersion::Version0,
        Some(stored_number) => SchemaVersion::from_int(stored_number).ok_or_else(|| {
            DbError::BadCode(format!(
                "web_migrations/main_for_bringing_the_db_to_the_newest_schema_version.rs, bring_the_db_to_the_newest_schema_version: stored schema version {stored_number} is not a known version"
            ))
        })?,
    };

    let newest_version = update_until_newest_version(version_the_db_is_on).await?;
    if stored_version != Some(newest_version.to_int()) {
        write_schema_version(newest_version.to_int()).await?;
    }
    Ok(())
}
