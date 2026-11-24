use protocol::error::DbError;
use protocol::new_table::ColumnDef;
use protocol::payload::CreateTableIn;
use web_internal_db::db_helper;

#[derive(Clone)]
pub struct ADbTable {
    pub table_name: String,
    pub columns: Vec<ColumnDef>,
}

pub async fn create_table_from_scratch(db_tables: Vec<ADbTable>) -> Result<(), DbError> {
    for db_table in db_tables {
        db_helper::create_table(CreateTableIn {
            table_name: db_table.table_name,
            columns: db_table.columns,
        })
        .await?;
    }
    Ok(())
}
