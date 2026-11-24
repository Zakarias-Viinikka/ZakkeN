use protocol::error::DbError;
use protocol::payload::{CreateFts5TableIn, RebuildFts5In};
use web_internal_db::db_helper;

use crate::web_migrations::helpers::ADbTable;
use crate::web_migrations::schemas::version0;

const SOURCE_TABLE_NAME: &str = "every_block_in_existence";
const INDEXED_COLUMN_NAME: &str = "content";

pub fn entire_table() -> Vec<ADbTable> {
    version0::entire_table()
}

pub async fn create_fts_table() -> Result<(), DbError> {
    db_helper::create_fts5_table(CreateFts5TableIn {
        source_table_name: SOURCE_TABLE_NAME.to_string(),
        columns: vec![INDEXED_COLUMN_NAME.to_string()],
    })
    .await
}

pub async fn update_from_old_version() -> Result<(), DbError> {
    create_fts_table().await?;
    db_helper::rebuild_fts5_index(RebuildFts5In {
        table_name: SOURCE_TABLE_NAME.to_string(),
    })
    .await
}
