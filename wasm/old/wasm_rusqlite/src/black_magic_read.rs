use crate::create_sql_statements::*;
use anyhow::Result;
use rusqlite::Connection;

pub fn read_from_db(
    conn: &Connection,
    table_name: impl AsRef<str>,
    arguments: &[impl AsRef<str>],
    columns_to_read: &[impl AsRef<str>],
) -> Result<Vec<Vec<String>>> {
    let sql = generate_read_from_table_sql(&table_name, arguments, columns_to_read);
    query_strings(conn, &sql)
}

pub fn read_from_db_ordered(
    conn: &Connection,
    table_name: impl AsRef<str>,
    arguments: &[impl AsRef<str>],
    columns_to_read: &[impl AsRef<str>],
    order_by: &str,
) -> Result<Vec<Vec<String>>> {
    let sql = generate_get_data_by_order_sql(&table_name, arguments, columns_to_read, order_by);
    query_strings(conn, &sql)
}

fn query_strings(conn: &Connection, sql: &str) -> Result<Vec<Vec<String>>> {
    let mut stmt = conn.prepare(sql)?;
    let col_count = stmt.column_count();

    let mut rows = Vec::new();
    let mut query = stmt.query([])?;

    while let Some(row) = query.next()? {
        let mut out = Vec::with_capacity(col_count);

        for i in 0..col_count {
            let value = match row.get_ref(i)? {
                rusqlite::types::ValueRef::Null => String::new(),
                rusqlite::types::ValueRef::Integer(n) => n.to_string(),
                rusqlite::types::ValueRef::Real(f) => f.to_string(),
                rusqlite::types::ValueRef::Text(t) => String::from_utf8_lossy(t).into_owned(),
                rusqlite::types::ValueRef::Blob(b) => String::from_utf8_lossy(b).into_owned(),
            };

            out.push(value);
        }

        rows.push(out);
    }

    Ok(rows)
}
