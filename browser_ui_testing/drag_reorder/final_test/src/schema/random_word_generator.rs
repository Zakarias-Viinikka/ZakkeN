use protocol::error::DbError;
use protocol::payload::CountAllRowsIn;
use random_word::{self, Lang};
use web_internal_db::db_helper;

use super::table::{BLOCKS_TABLE_NAME, PAGES_TABLE_NAME};

pub fn generate_id(prefix: &str) -> String {
    let mut result = prefix.to_string();
    for _ in 0..8 {
        let r = js_sys::Math::random();
        let idx = (r * 36.0) as u32;
        let c = if idx < 10 {
            (b'0' + idx as u8) as char
        } else {
            (b'a' + (idx - 10) as u8) as char
        };
        result.push(c);
    }
    result
}

pub async fn table_is_empty(table_name: &str) -> Result<bool, DbError> {
    let out = db_helper::count_all_rows(CountAllRowsIn {
        table_name: table_name.to_string(),
    })
    .await?;
    Ok(out.count == 0)
}

pub async fn table_exists(table_name: &str) -> Result<bool, DbError> {
    let tables = db_helper::list_tables().await?.table_names;
    Ok(tables.contains(&table_name.to_string()))
}
