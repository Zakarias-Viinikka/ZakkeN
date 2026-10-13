pub mod table;
pub mod random_word_generator;
pub mod migration;

use protocol::error::DbError;
use protocol::payload::{ColumnValue, CreateTableIn, GetDataIn, InsertDataIn, SelectArgument, SelectArguments};
use protocol::row_col::Col;
use web_internal_db::db_helper;

use crate::schema::migration::helpers::{create_table_from_scratch, drop_all_tables, ADbTable};
use crate::schema::migration::schema_versions::{CURRENT_VERSION, SchemaVersion};
use crate::schema::migration::schemas::current;
use crate::schema::migration::update_from_old_schema::update_until_newest_version;
use crate::schema::random_word_generator::{generate_id, table_exists, table_is_empty};
use crate::schema::table::{all_tables, BLOCKS_TABLE_NAME, PAGES_TABLE_NAME};
use crate::shared_structs::{BlockHolder, PageHolder};

const VERSION_TABLE: &str = "schema_version";

pub async fn init_db() -> Result<(), DbError> {
    ensure_version_table().await?;

    let pages_exist = table_exists(PAGES_TABLE_NAME).await?;
    let blocks_exist = table_exists(BLOCKS_TABLE_NAME).await?;
    let blocks_empty = if blocks_exist {
        table_is_empty(BLOCKS_TABLE_NAME).await?
    } else {
        true
    };

    let is_fresh = !pages_exist || !blocks_exist || blocks_empty;

    if is_fresh {
        let mut drop_tables: Vec<ADbTable> = Vec::new();
        if blocks_exist {
            drop_tables.push(ADbTable {
                table_name: BLOCKS_TABLE_NAME.to_string(),
                columns: Vec::new(),
                foreign_keys: Vec::new(),
            });
        }
        if pages_exist {
            drop_tables.push(ADbTable {
                table_name: PAGES_TABLE_NAME.to_string(),
                columns: Vec::new(),
                foreign_keys: Vec::new(),
            });
        }
        if !drop_tables.is_empty() {
            drop_all_tables(&drop_tables).await?;
        }
        
        create_table_from_scratch(current::entire_table()).await?;
        write_version(CURRENT_VERSION.to_int()).await?;
        seed_data().await?;
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
        columns: vec![
            protocol::new_table::id_column(),
            protocol::new_table::not_null_col(protocol::new_table::ColumnType::Integer, "version"),
        ],
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
        db_helper::edit_col_in_row(protocol::payload::EditColInRowIn {
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

async fn seed_data() -> Result<(), DbError> {
    let word_counts = [2, 9, 4, 20, 6];
    let mut page_ids = Vec::new();
    
    for _ in 0..2 {
        let page_id = generate_id("pg_");
        page_ids.push(page_id.clone());
        db_helper::insert_data(InsertDataIn {
            table_name: PAGES_TABLE_NAME.to_string(),
            values: vec![ColumnValue {
                column_name: "page_id".to_string(),
                value: Col::Text(page_id),
            }],
        })
        .await?;
    }

    for page_id in page_ids.iter() {
        for i in 0..5 {
            let block_id = generate_id("bl_");
            let count = word_counts[i % word_counts.len()];
            let mut content = String::new();
            for _ in 0..count {
                content.push_str(&random_word::get(random_word::Lang::En));
                content.push(' ');
            }
            
            db_helper::insert_data(InsertDataIn {
                table_name: BLOCKS_TABLE_NAME.to_string(),
                values: vec![
                    ColumnValue {
                        column_name: "block_id".to_string(),
                        value: Col::Text(block_id),
                    },
                    ColumnValue {
                        column_name: "page_id".to_string(),
                        value: Col::Text(page_id.clone()),
                    },
                    ColumnValue {
                        column_name: "content".to_string(),
                        value: Col::Text(content.trim().to_string()),
                    },
                ],
            })
            .await?;
        }
    }
    Ok(())
}

pub async fn get_all_pages() -> Result<Vec<PageHolder>, DbError> {
    let out = db_helper::get_data(GetDataIn {
        table_name: PAGES_TABLE_NAME.to_string(),
        arguments: SelectArguments::Single(SelectArgument::All),
        columns_to_read: vec!["id".to_string(), "page_id".to_string()],
    })
    .await?;

    let mut result = Vec::new();
    for row in out.rows {
        let mut cols = row.cols.into_iter();
        let id_col = cols.next().ok_or_else(|| DbError::ColDestructFail("missing id column".into()))?;
        let page_id_col = cols.next().ok_or_else(|| DbError::ColDestructFail("missing page_id column".into()))?;
        
        let db_id = match id_col {
            Col::Integer(v) => v as usize,
            _ => return Err(DbError::ColDestructFail("id is not an integer".into())),
        };
        let page_id = match page_id_col {
            Col::Text(s) => s,
            _ => return Err(DbError::ColDestructFail("page_id is not text".into())),
        };
        result.push(PageHolder { db_id, page_id });
    }
    result.sort_by_key(|p| p.db_id);
    Ok(result)
}

pub async fn get_all_blocks() -> Result<Vec<BlockHolder>, DbError> {
    let out = db_helper::get_data(GetDataIn {
        table_name: BLOCKS_TABLE_NAME.to_string(),
        arguments: SelectArguments::Single(SelectArgument::All),
        columns_to_read: vec!["id".to_string(), "block_id".to_string(), "page_id".to_string(), "content".to_string()],
    })
    .await?;

    let mut result = Vec::new();
    for row in out.rows {
        let mut cols = row.cols.into_iter();
        let id_col = cols.next().ok_or_else(|| DbError::ColDestructFail("missing id column".into()))?;
        let block_id_col = cols.next().ok_or_else(|| DbError::ColDestructFail("missing block_id column".into()))?;
        let page_id_col = cols.next().ok_or_else(|| DbError::ColDestructFail("missing page_id column".into()))?;
        let content_col = cols.next().ok_or_else(|| DbError::ColDestructFail("missing content column".into()))?;
        
        let db_id = match id_col {
            Col::Integer(v) => v as usize,
            _ => return Err(DbError::ColDestructFail("id is not an integer".into())),
        };
        let block_id = match block_id_col {
            Col::Text(s) => s,
            _ => return Err(DbError::ColDestructFail("block_id is not text".into())),
        };
        let page_id = match page_id_col {
            Col::Text(s) => s,
            _ => return Err(DbError::ColDestructFail("page_id is not text".into())),
        };
        let content = match content_col {
            Col::Text(s) => s,
            _ => return Err(DbError::ColDestructFail("content is not text".into())),
        };
        result.push(BlockHolder { db_id, block_id, page_id, content });
    }
    result.sort_by_key(|b| b.db_id);
    Ok(result)
}
