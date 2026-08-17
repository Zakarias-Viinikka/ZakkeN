use crate::public_data_shapes;
use crate::table_row::{Col, Row};
use crate::{create_sql_statements::*, public_data_shapes::DbError};
use anyhow::Result;
use rusqlite::Connection;

pub fn read_from_db(
    conn: &Connection,
    ctx: &public_data_shapes::GetDataIn,
) -> Result<Vec<Row>, DbError> {
    let table_name = &ctx.table_name;
    let arguments: Vec<String> = ctx.arguments.iter().map(|x| x.to_sql_condition()).collect();
    let columns_to_read = &ctx.columns_to_read;
    let sql = generate_read_from_table_sql(table_name, &arguments, columns_to_read);
    query_rows(conn, &sql)
        .map_err(|e| DbError::SqlExecuteFail(format!("read_from_db failed: {:?}, sql: {}", e, sql)))
}

pub fn read_from_db_ordered(
    conn: &Connection,
    ctx: &public_data_shapes::GetDataOrderedIn,
) -> Result<Vec<Row>, DbError> {
    let table_name = &ctx.table_name;
    let arguments: Vec<String> = ctx.arguments.iter().map(|x| x.to_sql_condition()).collect();
    let columns_to_read = &ctx.columns_to_read;
    let order_by = &ctx.order_by;

    let sql = generate_get_data_by_order_sql(table_name, &arguments, columns_to_read, order_by);
    query_rows(conn, &sql)
}

fn query_rows(conn: &Connection, sql: &str) -> Result<Vec<Row>, DbError> {
    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| DbError::SqlExecuteFail(e.to_string()))?;
    let col_count = stmt.column_count();
    let mut rows = Vec::new();
    let mut query = stmt
        .query([])
        .map_err(|e| DbError::SqlExecuteFail(e.to_string()))?;

    while let Some(row) = query
        .next()
        .map_err(|e| DbError::SqlExecuteFail(e.to_string()))?
    {
        let mut cols = Vec::with_capacity(col_count);
        for i in 0..col_count {
            let col = match row
                .get_ref(i)
                .map_err(|e| DbError::SqlExecuteFail(e.to_string()))?
            {
                rusqlite::types::ValueRef::Null => Col::Null,
                rusqlite::types::ValueRef::Integer(n) => Col::Integer(n),
                rusqlite::types::ValueRef::Real(f) => Col::Real(f),
                rusqlite::types::ValueRef::Text(t) => {
                    Col::Text(String::from_utf8_lossy(t).into_owned())
                }
                rusqlite::types::ValueRef::Blob(b) => Col::Blob(b.to_vec()),
            };
            cols.push(col);
        }
        rows.push(Row { cols });
    }
    Ok(rows)
}
