use protocol::error::DbError;
use protocol::new_table::{ColumnDef, ForeignKeyDef};
use protocol::payload::{CheckTableIn, CreateForeignTableIn, CreateTableIn, DropTableIn};
use web_internal_db::db_helper;

#[derive(Clone)]
pub struct ADbTable {
    pub table_name: String,
    pub columns: Vec<ColumnDef>,
    pub foreign_keys: Vec<ForeignKeyDef>,
}

pub async fn create_table_from_scratch(db_tables: Vec<ADbTable>) -> Result<(), DbError> {
    for db_table in db_tables {
        if db_table.foreign_keys.is_empty() {
            db_helper::create_table(CreateTableIn {
                table_name: db_table.table_name,
                columns: db_table.columns,
            })
            .await?;
        } else {
            db_helper::create_foreign_table(CreateForeignTableIn {
                table_name: db_table.table_name,
                columns: db_table.columns,
                foreign_keys: db_table.foreign_keys,
            })
            .await?;
        }
    }
    Ok(())
}

pub async fn drop_all_tables(db_tables: &[ADbTable]) -> Result<(), DbError> {
    for t in db_tables {
        db_helper::force_drop_table(DropTableIn {
            table_name: t.table_name.clone(),
        })
        .await?;
    }
    Ok(())
}

pub async fn table_shapes(
    db_tables: &[ADbTable],
) -> Result<Vec<protocol::payload::CheckTableOut>, DbError> {
    let mut out = Vec::new();
    for t in db_tables {
        out.push(
            db_helper::check_table(CheckTableIn {
                table_name: t.table_name.clone(),
            })
            .await?,
        );
    }
    Ok(out)
}
