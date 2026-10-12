use protocol::error::DbError;
use protocol::payload::{CountRowsIn, SelectArgument, SelectArguments};
use random_word::{self, Lang};
use web_internal_db::db_helper;

use super::table::{TABLE_NAME, WORD};

pub fn generate_word() -> String {
    random_word::get(Lang::En).to_string()
}

pub async fn word_already_exists(word: &str) -> Result<bool, DbError> {
    let out = db_helper::count_rows(CountRowsIn {
        table_name: TABLE_NAME.to_string(),
        arguments: SelectArguments::Single(SelectArgument::XEqualY {
            x: WORD.name.to_string(),
            y: word.to_string(),
        }),
    })
    .await?;
    Ok(out.count > 0)
}