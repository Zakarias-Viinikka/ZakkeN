pub mod table;
pub mod random_word_generator;
pub mod migration;

use protocol::error::DbError;
use protocol::new_table::{ColumnType, id_column, not_null_col};
use protocol::payload::{
    ColumnValue, CountAllRowsIn, CreateTableIn, EditColInRowIn, GetDataIn, InsertDataIn,
    SelectArgument, SelectArguments,
};
use protocol::row_col::Col;
use web_internal_db::db_helper;

use crate::schema::migration::helpers::create_table_from_scratch;
use crate::schema::migration::schema_versions::{CURRENT_VERSION, SchemaVersion};
use crate::schema::migration::schemas::current;
use crate::schema::migration::update_from_old_schema::update_until_newest_version;
use crate::schema::random_word_generator::{generate_word, word_already_exists};
use crate::schema::table::{new_word_row, TABLE_NAME, WORD};
use crate::shared_structs::WordHolder;

const VERSION_TABLE: &str = "schema_version";

pub async fn init_db() -> Result<(), DbError> {
    ensure_version_table().await?;

    let tables = db_helper::list_tables().await?.table_names;
    let is_fresh = !tables.contains(&TABLE_NAME.to_string());

    if is_fresh {
        create_table_from_scratch(current::entire_table()).await?;
        write_version(CURRENT_VERSION.to_int()).await?;
    } else {
        let stored = read_version().await?;
        let current = stored
            .and_then(SchemaVersion::from_int)
            .unwrap_or(SchemaVersion::Version0);

        if current != CURRENT_VERSION {
            let new_v = update_until_newest_version(current).await?;
            write_version(new_v.to_int()).await?;
        }
    }

    seed_five_words().await?;
    Ok(())
}

async fn ensure_version_table() -> Result<(), DbError> {
    if db_helper::list_tables()
        .await?
        .table_names
        .contains(&VERSION_TABLE.to_string())
    {
        return Ok(());
    }
    db_helper::create_table(CreateTableIn {
        table_name: VERSION_TABLE.to_string(),
        columns: vec![id_column(), not_null_col(ColumnType::Integer, "version")],
    })
    .await
}

async fn read_version() -> Result<Option<i64>, DbError> {
    let out = db_helper::get_data(GetDataIn {
        table_name: VERSION_TABLE.to_string(),
        arguments: SelectArguments::Single(SelectArgument::All),
        columns_to_read: vec!["version".to_string()],
    })
    .await?;

    let Some(row) = out.rows.into_iter().next() else {
        return Ok(None);
    };
    let Some(col) = row.cols.into_iter().next() else {
        return Ok(None);
    };
    match col {
        Col::Integer(v) => Ok(Some(v)),
        _ => Err(DbError::ColDestructFail(
            "schema_version.version is not an integer".into(),
        )),
    }
}

async fn write_version(v: i64) -> Result<(), DbError> {
    if read_version().await?.is_some() {
        db_helper::edit_col_in_row(EditColInRowIn {
            table_name: VERSION_TABLE.to_string(),
            row_id: "1".to_string(),
            column: "version".to_string(),
            new_value: Col::Integer(v),
        })
        .await
    } else {
        db_helper::insert_data(InsertDataIn {
            table_name: VERSION_TABLE.to_string(),
            values: vec![ColumnValue {
                column_name: "version".to_string(),
                value: Col::Integer(v),
            }],
        })
        .await
    }
}

async fn seed_five_words() -> Result<(), DbError> {
    let have = db_helper::count_all_rows(CountAllRowsIn {
        table_name: TABLE_NAME.to_string(),
    })
    .await?
    .count;

    let mut need = 5i64 - have as i64;
    while need > 0 {
        let word = generate_word();
        if word_already_exists(&word).await? {
            continue;
        }
        db_helper::insert_data(InsertDataIn {
            table_name: TABLE_NAME.to_string(),
            values: new_word_row(word),
        })
        .await?;
        need -= 1;
    }
    Ok(())
}

pub async fn get_all_words() -> Result<Vec<WordHolder>, DbError> {
    let out = db_helper::get_data(GetDataIn {
        table_name: TABLE_NAME.to_string(),
        arguments: SelectArguments::Single(SelectArgument::All),
        columns_to_read: vec!["id".to_string(), WORD.name.to_string()],
    })
    .await?;

    let mut result = Vec::new();
    for row in out.rows {
        let mut cols = row.cols.into_iter();
        let id_col = cols
            .next()
            .ok_or_else(|| DbError::ColDestructFail("missing id column".into()))?;
        let word_col = cols
            .next()
            .ok_or_else(|| DbError::ColDestructFail("missing word column".into()))?;
        let db_id = match id_col {
            Col::Integer(v) => v as usize,
            _ => return Err(DbError::ColDestructFail("id is not an integer".into())),
        };
        let word = match word_col {
            Col::Text(s) => s,
            _ => return Err(DbError::ColDestructFail("word is not text".into())),
        };
        result.push(WordHolder { db_id, word });
    }
    Ok(result)
}