use crate::create_table::ColumnDef;

// Builds CREATE TABLE SQL from a caller-supplied column list (replaces the old Table/Column version).
pub fn generate_create_table_sql(table_name: &str, columns: &[ColumnDef]) -> String {
    let mut col_defs = Vec::new();
    for col in columns {
        let mut def = format!("{} {}", quote_ident(&col.0), col.1);
        if col.2 {
            def.push_str(" PRIMARY KEY");
        }
        if col.6 {
            def.push_str(" AUTOINCREMENT");
        }
        if col.3 {
            def.push_str(" NOT NULL");
        }
        if col.4 {
            def.push_str(" UNIQUE");
        }
        if !col.5.is_empty() {
            def.push_str(&format!(" DEFAULT {}", col.5));
        }
        col_defs.push(def);
    }
    format!(
        "CREATE TABLE IF NOT EXISTS {} ({});",
        quote_ident(table_name),
        col_defs.join(", ")
    )
}

pub fn generate_add_column_sql(table_name: &str, column: &ColumnDef) -> String {
    let mut def = format!("{} {}", quote_ident(&column.0), column.1);
    if column.2 {
        def.push_str(" PRIMARY KEY");
    }
    if column.6 {
        def.push_str(" AUTOINCREMENT");
    }
    if column.3 {
        def.push_str(" NOT NULL");
    }
    if column.4 {
        def.push_str(" UNIQUE");
    }
    if !column.5.is_empty() {
        def.push_str(&format!(" DEFAULT {}", column.5));
    }

    format!(
        "ALTER TABLE {} ADD COLUMN {};",
        quote_ident(table_name),
        def
    )
}

pub fn generate_drop_column_sql(table_name: &str, column_name: &str) -> String {
    format!(
        "ALTER TABLE {} DROP COLUMN {};",
        quote_ident(table_name),
        quote_ident(column_name)
    )
}

// Builds INSERT SQL from (column, value) pairs - replaces the old positional
// Table/Vec<String> version. Order comes from the pairs themselves, not two
// separately-ordered lists, so columns and values can't drift apart.
pub fn generate_insert_sql(table_name: &str, values: Vec<(String, table_row::Col)>) -> String {
    let columns: Vec<String> = values.iter().map(|(col, _)| quote_ident(col)).collect();

    let quoted_values: Vec<String> = values
        .iter()
        .map(|(_, value)| col_to_sql_literal(value))
        .collect();

    format!(
        "INSERT INTO {} ({}) VALUES ({});",
        quote_ident(table_name),
        columns.join(", "),
        quoted_values.join(", ")
    )
}

pub fn generate_delete_sql(table_name: &str, id: &str) -> String {
    format!("DELETE FROM {} WHERE id = {};", quote_ident(table_name), id)
}

pub fn generate_update_sql_typed(
    table_name: &str,
    id: usize,
    column: &str,
    new_value: &table_row::Col,
) -> String {
    let quoted_table = quote_ident(table_name);
    let quoted_column = quote_ident(column);
    let value_literal = col_to_sql_literal(new_value);

    let col_and_val = format!("{} = {}", quoted_column, value_literal);

    format!(
        "UPDATE {table} SET {col_and_val} WHERE id = {id};",
        table = quoted_table,
        col_and_val = col_and_val,
        id = id
    )
}

pub fn generate_read_from_table_sql(
    table_name: impl AsRef<str>,
    arguments: &[impl AsRef<str>],
    columns_to_read: &[impl AsRef<str>],
) -> String {
    let valid_columns: Vec<&str> = columns_to_read
        .iter()
        .filter(|c| !c.as_ref().is_empty())
        .map(|c| c.as_ref())
        .collect();

    let columns = if valid_columns.is_empty() {
        "*".to_string()
    } else {
        valid_columns
            .iter()
            .map(|c| quote_ident(c))
            .collect::<Vec<_>>()
            .join(", ")
    };

    let valid_conditions: Vec<&str> = arguments
        .iter()
        .filter(|a| !a.as_ref().is_empty())
        .map(|a| a.as_ref())
        .collect();

    let where_clause = if valid_conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", valid_conditions.join(" AND "))
    };

    format!(
        "SELECT {} FROM {}{};",
        columns,
        quote_ident(table_name.as_ref()),
        where_clause
    )
}

pub fn generate_get_data_by_order_sql(
    table_name: impl AsRef<str>,
    arguments: &[impl AsRef<str>],
    columns_to_read: &[impl AsRef<str>],
    order_by: &str,
) -> String {
    let valid_columns: Vec<&str> = columns_to_read
        .iter()
        .filter(|c| !c.as_ref().is_empty())
        .map(|c| c.as_ref())
        .collect();

    let columns = if valid_columns.is_empty() {
        "*".to_string()
    } else {
        valid_columns
            .iter()
            .map(|c| quote_ident(c))
            .collect::<Vec<_>>()
            .join(", ")
    };

    let valid_conditions: Vec<&str> = arguments
        .iter()
        .filter(|a| !a.as_ref().is_empty())
        .map(|a| a.as_ref())
        .collect();

    let where_clause = if valid_conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", valid_conditions.join(" AND "))
    };

    format!(
        "SELECT {} FROM {}{} ORDER BY {};",
        columns,
        quote_ident(table_name.as_ref()),
        where_clause,
        order_by
    )
}

use crate::table_row;
fn col_to_sql_literal(value: &table_row::Col) -> String {
    match value {
        table_row::Col::Null => "NULL".to_string(),
        table_row::Col::Integer(i) => i.to_string(),
        table_row::Col::Real(f) => f.to_string(),
        table_row::Col::Text(s) => format!("'{}'", sanitize(s)),
        table_row::Col::Blob(bytes) => {
            let hex: String = bytes.iter().map(|b| format!("{:02X}", b)).collect();
            format!("X'{}'", hex)
        }
    }
}

fn sanitize(input: &str) -> String {
    input.replace("'", "''")
}

pub fn quote_ident(ident: &str) -> String {
    format!("\"{}\"", ident.replace('"', "\"\""))
}

pub fn quote_sql_string(s: &str) -> String {
    format!("'{}'", s.replace('\'', "''"))
}

/*
enum HappySql { SanitizedSqlInput(String)) }

together with a method i would have like fn sanitize_userinput_to_sql(input: String) -> SanitizedSqlInput(String)) {}
 */

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;

    // =========================================================================
    //  generate_create_table_sql
    // =========================================================================

    mod create_table_dynamic_tests {
        use super::*;

        fn col(
            name: &str,
            col_type: &str,
            pk: bool,
            not_null: bool,
            unique: bool,
            default: &str,
            autoinc: bool,
        ) -> ColumnDef {
            ColumnDef(
                name.to_string(),
                col_type.to_string(),
                pk,
                not_null,
                unique,
                default.to_string(),
                autoinc,
            )
        }

        #[wasm_bindgen_test]
        fn plain_column_no_constraints() {
            let columns = vec![col("name", "TEXT", false, false, false, "", false)];
            let sql = generate_create_table_sql("users", &columns);
            assert_eq!(sql, "CREATE TABLE IF NOT EXISTS \"users\" (\"name\" TEXT);");
        }

        #[wasm_bindgen_test]
        fn primary_key_alone_without_autoincrement() {
            let columns = vec![col("id", "INTEGER", true, false, false, "", false)];
            let sql = generate_create_table_sql("users", &columns);
            assert_eq!(
                sql,
                "CREATE TABLE IF NOT EXISTS \"users\" (\"id\" INTEGER PRIMARY KEY);"
            );
        }

        #[wasm_bindgen_test]
        fn primary_key_with_autoincrement() {
            let columns = vec![col("id", "INTEGER", true, false, false, "", true)];
            let sql = generate_create_table_sql("users", &columns);
            assert_eq!(
                sql,
                "CREATE TABLE IF NOT EXISTS \"users\" (\"id\" INTEGER PRIMARY KEY AUTOINCREMENT);"
            );
        }

        #[wasm_bindgen_test]
        fn autoincrement_without_primary_key_does_not_appear() {
            let columns = vec![col("id", "INTEGER", false, false, false, "", true)];
            let sql = generate_create_table_sql("users", &columns);
            assert_eq!(
                sql,
                "CREATE TABLE IF NOT EXISTS \"users\" (\"id\" INTEGER AUTOINCREMENT);"
            );
        }

        #[wasm_bindgen_test]
        fn not_null_flag() {
            let columns = vec![col("email", "TEXT", false, true, false, "", false)];
            let sql = generate_create_table_sql("users", &columns);
            assert_eq!(
                sql,
                "CREATE TABLE IF NOT EXISTS \"users\" (\"email\" TEXT NOT NULL);"
            );
        }

        #[wasm_bindgen_test]
        fn unique_flag() {
            let columns = vec![col("email", "TEXT", false, false, true, "", false)];
            let sql = generate_create_table_sql("users", &columns);
            assert_eq!(
                sql,
                "CREATE TABLE IF NOT EXISTS \"users\" (\"email\" TEXT UNIQUE);"
            );
        }

        #[wasm_bindgen_test]
        fn default_value_present() {
            let columns = vec![col("status", "TEXT", false, false, false, "active", false)];
            let sql = generate_create_table_sql("users", &columns);
            assert_eq!(
                sql,
                "CREATE TABLE IF NOT EXISTS \"users\" (\"status\" TEXT DEFAULT active);"
            );
        }

        #[wasm_bindgen_test]
        fn empty_default_string_omits_default_clause() {
            let columns = vec![col("status", "TEXT", false, false, false, "", false)];
            let sql = generate_create_table_sql("users", &columns);
            assert!(!sql.contains("DEFAULT"));
        }

        #[wasm_bindgen_test]
        fn all_constraints_combined_in_correct_order() {
            let columns = vec![col("id", "INTEGER", true, true, true, "1", true)];
            let sql = generate_create_table_sql("users", &columns);
            assert_eq!(
                sql,
                "CREATE TABLE IF NOT EXISTS \"users\" (\"id\" INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL UNIQUE DEFAULT 1);"
            );
        }

        #[wasm_bindgen_test]
        fn multiple_columns_joined_with_commas() {
            let columns = vec![
                col("id", "INTEGER", true, false, false, "", true),
                col("name", "TEXT", false, true, false, "", false),
                col("age", "INTEGER", false, false, false, "0", false),
            ];
            let sql = generate_create_table_sql("people", &columns);
            assert_eq!(
                sql,
                "CREATE TABLE IF NOT EXISTS \"people\" (\"id\" INTEGER PRIMARY KEY AUTOINCREMENT, \"name\" TEXT NOT NULL, \"age\" INTEGER DEFAULT 0);"
            );
        }

        #[wasm_bindgen_test]
        fn table_name_is_not_hardcoded() {
            let columns = vec![col("x", "TEXT", false, false, false, "", false)];
            let sql_a = generate_create_table_sql("alpha", &columns);
            let sql_b = generate_create_table_sql("beta", &columns);
            assert!(sql_a.contains("\"alpha\""));
            assert!(sql_b.contains("\"beta\""));
            assert_ne!(sql_a, sql_b);
        }

        #[wasm_bindgen_test]
        fn empty_column_list_produces_empty_parens() {
            let sql = generate_create_table_sql("empty_table", &[]);
            assert_eq!(sql, "CREATE TABLE IF NOT EXISTS \"empty_table\" ();");
        }
    }

    // =========================================================================
    //  generate_insert_sql
    // =========================================================================

    #[wasm_bindgen_test]
    pub fn test_generate_insert_sql() {
        use crate::table_row::Col;

        let values = vec![
            ("product_id".to_string(), Col::Text("100".to_string())),
            ("product_name".to_string(), Col::Text("Laptop".to_string())),
        ];
        let sql = generate_insert_sql("products", values);
        let expected =
            "INSERT INTO \"products\" (\"product_id\", \"product_name\") VALUES ('100', 'Laptop');";
        assert_eq!(sql, expected);
    }

    #[wasm_bindgen_test]
    pub fn test_generate_insert_sql_escapes_quotes() {
        use crate::table_row::Col;

        let values = vec![("name".to_string(), Col::Text("O'Reilly".to_string()))];
        let sql = generate_insert_sql("authors", values);
        let expected = "INSERT INTO \"authors\" (\"name\") VALUES ('O''Reilly');";
        assert_eq!(sql, expected);
    }

    #[wasm_bindgen_test]
    pub fn test_generate_insert_sql_table_name_is_not_hardcoded() {
        use crate::table_row::Col;

        let values = vec![("x".to_string(), Col::Text("1".to_string()))];
        let sql_a = generate_insert_sql("alpha", values.clone());
        let sql_b = generate_insert_sql("beta", values);
        assert!(sql_a.contains("\"alpha\""));
        assert!(sql_b.contains("\"beta\""));
        assert_ne!(sql_a, sql_b);
    }

    // =========================================================================
    //  sanitize
    // =========================================================================

    #[wasm_bindgen_test]
    pub fn test_sanitize_no_quotes() {
        assert_eq!(sanitize("hello"), "hello");
    }

    #[wasm_bindgen_test]
    pub fn test_sanitize_single_quote() {
        assert_eq!(sanitize("O'Reilly"), "O''Reilly");
    }

    #[wasm_bindgen_test]
    pub fn test_sanitize_multiple_quotes() {
        assert_eq!(sanitize("a'b'c"), "a''b''c");
    }

    #[wasm_bindgen_test]
    pub fn test_sanitize_empty_string() {
        assert_eq!(sanitize(""), "");
    }

    #[wasm_bindgen_test]
    pub fn test_sanitize_quote_at_start_and_end() {
        assert_eq!(sanitize("'abc'"), "''abc''");
    }

    // =========================================================================
    //  generate_read_from_table_sql
    // =========================================================================

    #[wasm_bindgen_test]
    fn test_select_all_columns_no_conditions() {
        let sql = generate_read_from_table_sql("players", &[] as &[&str], &[] as &[&str]);
        assert_eq!(sql, "SELECT * FROM \"players\";");
    }

    #[wasm_bindgen_test]
    fn single_empty_string_column_becomes_star() {
        let sql = generate_read_from_table_sql("content", &["id = 1"], &[""]);
        assert_eq!(sql, "SELECT * FROM \"content\" WHERE id = 1;");
    }

    #[wasm_bindgen_test]
    fn empty_string_mixed_with_real_column_should_be_filtered() {
        let sql = generate_read_from_table_sql("content", &["id = 1"], &["col1", ""]);
        assert_eq!(sql, "SELECT \"col1\" FROM \"content\" WHERE id = 1;");
    }

    #[wasm_bindgen_test]
    fn empty_arguments_vec_omits_where() {
        let sql = generate_read_from_table_sql("content", &[] as &[&str], &["col1", "col2"]);
        assert_eq!(sql, "SELECT \"col1\", \"col2\" FROM \"content\";");
    }

    #[wasm_bindgen_test]
    fn single_empty_string_argument_omits_where() {
        let sql = generate_read_from_table_sql("content", &[""], &["col1"]);
        assert_eq!(sql, "SELECT \"col1\" FROM \"content\";");
    }

    #[wasm_bindgen_test]
    fn both_empty_gives_select_star_no_where() {
        let sql = generate_read_from_table_sql("content", &[""], &[""]);
        assert_eq!(sql, "SELECT * FROM \"content\";");
    }

    #[wasm_bindgen_test]
    fn two_empty_string_arguments_should_omit_where() {
        let sql = generate_read_from_table_sql("content", &["", ""], &[""]);
        assert_eq!(sql, "SELECT * FROM \"content\";");
    }

    #[wasm_bindgen_test]
    fn empty_string_mixed_with_real_argument_should_be_filtered() {
        let sql = generate_read_from_table_sql("content", &["", "id = 1"], &["col1"]);
        assert_eq!(sql, "SELECT \"col1\" FROM \"content\" WHERE id = 1;");
    }

    #[wasm_bindgen_test]
    fn test_select_specific_columns_with_condition() {
        let sql = generate_read_from_table_sql("games", &["result = '1-0'"], &["white", "black"]);
        assert_eq!(
            sql,
            "SELECT \"white\", \"black\" FROM \"games\" WHERE result = '1-0';"
        );
    }

    #[wasm_bindgen_test]
    fn test_multiple_conditions_multiple_columns() {
        let sql = generate_read_from_table_sql(
            "puzzles",
            &["rating > 2000", "theme = 'mate'"],
            &["id", "fen", "solution"],
        );
        assert_eq!(
            sql,
            "SELECT \"id\", \"fen\", \"solution\" FROM \"puzzles\" WHERE rating > 2000 AND theme = 'mate';"
        );
    }

    #[wasm_bindgen_test]
    fn test_empty_columns_with_condition() {
        let sql =
            generate_read_from_table_sql("openings", &["name LIKE '%Sicilian%'"], &[] as &[&str]);
        assert_eq!(
            sql,
            "SELECT * FROM \"openings\" WHERE name LIKE '%Sicilian%';"
        );
    }

    #[wasm_bindgen_test]
    fn normal_case_still_works() {
        let sql = generate_read_from_table_sql("content", &["id = 1"], &["col1", "col2"]);
        assert_eq!(
            sql,
            "SELECT \"col1\", \"col2\" FROM \"content\" WHERE id = 1;"
        );
    }

    // =========================================================================
    //  generate_get_data_by_order_sql
    // =========================================================================

    #[wasm_bindgen_test]
    fn order_by_no_conditions_all_columns() {
        let sql =
            generate_get_data_by_order_sql("content", &[] as &[&str], &[] as &[&str], "position");
        assert_eq!(sql, "SELECT * FROM \"content\" ORDER BY position;");
    }

    #[wasm_bindgen_test]
    fn order_by_with_condition() {
        let sql =
            generate_get_data_by_order_sql("content", &["id = 1"], &[] as &[&str], "position");
        assert_eq!(
            sql,
            "SELECT * FROM \"content\" WHERE id = 1 ORDER BY position;"
        );
    }

    #[wasm_bindgen_test]
    fn order_by_with_specific_columns() {
        let sql = generate_get_data_by_order_sql(
            "content",
            &[] as &[&str],
            &["content", "position"],
            "position",
        );
        assert_eq!(
            sql,
            "SELECT \"content\", \"position\" FROM \"content\" ORDER BY position;"
        );
    }

    #[wasm_bindgen_test]
    fn order_by_with_columns_and_condition() {
        let sql =
            generate_get_data_by_order_sql("content", &["id = 1"], &["content"], "position DESC");
        assert_eq!(
            sql,
            "SELECT \"content\" FROM \"content\" WHERE id = 1 ORDER BY position DESC;"
        );
    }

    #[wasm_bindgen_test]
    fn order_by_empty_string_argument_omits_where() {
        let sql = generate_get_data_by_order_sql("content", &[""], &[""], "position");
        assert_eq!(sql, "SELECT * FROM \"content\" ORDER BY position;");
    }

    // =========================================================================
    //  generate_add_column_sql
    // =========================================================================

    mod add_column_sql_tests {
        use super::*;

        fn col(
            name: &str,
            col_type: &str,
            pk: bool,
            not_null: bool,
            unique: bool,
            default: &str,
            autoinc: bool,
        ) -> ColumnDef {
            ColumnDef(
                name.to_string(),
                col_type.to_string(),
                pk,
                not_null,
                unique,
                default.to_string(),
                autoinc,
            )
        }

        #[wasm_bindgen_test]
        fn test_add_column_basic() {
            let col_def = col("age", "INTEGER", false, false, false, "", false);
            let sql = generate_add_column_sql("users", &col_def);
            assert_eq!(sql, "ALTER TABLE \"users\" ADD COLUMN \"age\" INTEGER;");
        }

        #[wasm_bindgen_test]
        fn test_add_column_with_not_null() {
            let col_def = col("email", "TEXT", false, true, false, "", false);
            let sql = generate_add_column_sql("users", &col_def);
            assert_eq!(
                sql,
                "ALTER TABLE \"users\" ADD COLUMN \"email\" TEXT NOT NULL;"
            );
        }

        #[wasm_bindgen_test]
        fn test_add_column_with_default() {
            let col_def = col("status", "TEXT", false, false, false, "active", false);
            let sql = generate_add_column_sql("users", &col_def);
            assert_eq!(
                sql,
                "ALTER TABLE \"users\" ADD COLUMN \"status\" TEXT DEFAULT active;"
            );
        }

        #[wasm_bindgen_test]
        fn test_add_column_with_primary_key() {
            let col_def = col("id", "INTEGER", true, false, false, "", false);
            let sql = generate_add_column_sql("users", &col_def);
            assert_eq!(
                sql,
                "ALTER TABLE \"users\" ADD COLUMN \"id\" INTEGER PRIMARY KEY;"
            );
        }

        #[wasm_bindgen_test]
        fn test_add_column_with_primary_key_and_autoincrement() {
            let col_def = col("id", "INTEGER", true, false, false, "", true);
            let sql = generate_add_column_sql("users", &col_def);
            assert_eq!(
                sql,
                "ALTER TABLE \"users\" ADD COLUMN \"id\" INTEGER PRIMARY KEY AUTOINCREMENT;"
            );
        }

        #[wasm_bindgen_test]
        fn test_add_column_with_unique() {
            let col_def = col("username", "TEXT", false, false, true, "", false);
            let sql = generate_add_column_sql("users", &col_def);
            assert_eq!(
                sql,
                "ALTER TABLE \"users\" ADD COLUMN \"username\" TEXT UNIQUE;"
            );
        }

        #[wasm_bindgen_test]
        fn test_add_column_all_constraints_in_correct_order() {
            let col_def = col("account_id", "INTEGER", true, true, true, "0", true);
            let sql = generate_add_column_sql("accounts", &col_def);
            assert_eq!(
                sql,
                "ALTER TABLE \"accounts\" ADD COLUMN \"account_id\" INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL UNIQUE DEFAULT 0;"
            );
        }

        #[wasm_bindgen_test]
        fn test_add_column_quotes_identifiers() {
            let col_def = col("full\"name", "TEXT", false, false, false, "", false);
            let sql = generate_add_column_sql("user table", &col_def);
            assert_eq!(
                sql,
                "ALTER TABLE \"user table\" ADD COLUMN \"full\"\"name\" TEXT;"
            );
        }

        #[wasm_bindgen_test]
        fn test_add_column_table_name_is_keyword() {
            let col_def = col("order", "TEXT", false, false, false, "", false);
            let sql = generate_add_column_sql("order", &col_def);
            assert_eq!(sql, "ALTER TABLE \"order\" ADD COLUMN \"order\" TEXT;");
        }
    }

    // =========================================================================
    //  generate_drop_column_sql
    // =========================================================================

    mod drop_column_sql_tests {
        use super::*;

        #[wasm_bindgen_test]
        fn test_drop_column_basic() {
            let sql = generate_drop_column_sql("users", "age");
            assert_eq!(sql, "ALTER TABLE \"users\" DROP COLUMN \"age\";");
        }

        #[wasm_bindgen_test]
        fn test_drop_column_quotes_identifiers() {
            let sql = generate_drop_column_sql("user table", "full\"name");
            assert_eq!(
                sql,
                "ALTER TABLE \"user table\" DROP COLUMN \"full\"\"name\";"
            );
        }

        #[wasm_bindgen_test]
        fn test_drop_column_table_name_is_keyword() {
            let sql = generate_drop_column_sql("order", "order");
            assert_eq!(sql, "ALTER TABLE \"order\" DROP COLUMN \"order\";");
        }
    }
}
