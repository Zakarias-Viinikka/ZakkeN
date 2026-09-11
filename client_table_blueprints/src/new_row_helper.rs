use my_yrs_lib::yrs_backlinks::YrsBacklinks;
use my_yrs_lib::yrs_wrapper::create_bookmark_of_synced_state;
use my_yrs_lib::{BossOfYrs, YrsActivePages, YrsError};
use protocol::row_col::{Col, Row};
use std::sync::Arc;

#[uniffi::export]
pub fn new_page_row(
    page_id: String,
    is_main_menu_page: bool,
    user_id: String,
) -> Result<Row, YrsError> {
    let page_doc = Arc::new(BossOfYrs::new(user_id));
    let blobbed_page = Arc::clone(&page_doc).snapshot()?;

    let version = create_bookmark_of_synced_state(page_doc)?;

    let active_doc = Arc::new(YrsActivePages::new_empty());
    let page_status = active_doc.snapshot()?;

    Ok(Row {
        cols: vec![
            Col::Text(page_id),
            Col::Blob(blobbed_page),
            Col::Blob(page_status),
            Col::Blob(version),
            Col::Text(if is_main_menu_page {
                "true".to_string()
            } else {
                "false".to_string()
            }),
        ],
    })
}

#[uniffi::export]
pub fn new_every_block_in_existence_row(
    title: String,
    content: String,
    my_id_as_given_by_yrs: String,
    id_of_page_i_belong_to: String,
) -> Result<Row, YrsError> {
    Ok(Row {
        cols: vec![
            Col::Text(title),
            Col::Text(content),
            Col::Text(my_id_as_given_by_yrs),
            Col::Text(id_of_page_i_belong_to),
        ],
    })
}

#[uniffi::export]
pub fn new_uncommitted_diff_row(
    snapshot_of_edit: Vec<u8>,
    love_letter_sketch: Vec<u8>,
    session_id: String,
    target_id: String,
) -> Result<Row, YrsError> {
    Ok(Row {
        cols: vec![
            Col::Blob(snapshot_of_edit),
            Col::Blob(love_letter_sketch),
            Col::Text(session_id),
            Col::Text(target_id),
        ],
    })
}

#[uniffi::export]
pub fn new_backlink_row(
    page_that_holds_link_id: String,
    page_being_linked_to_id: String,
) -> Result<Row, YrsError> {
    let backlinks_doc = Arc::new(YrsBacklinks::new_empty());
    let disabled = backlinks_doc.clone().snapshot()?;
    let version = backlinks_doc.create_bookmark_of_synced_state()?;

    Ok(Row {
        cols: vec![
            Col::Text(page_that_holds_link_id),
            Col::Text(page_being_linked_to_id),
            Col::Blob(disabled),
            Col::Blob(version),
        ],
    })
}

#[uniffi::export]
pub fn new_key_value_item(key: String, value: String) -> Result<Row, YrsError> {
    Ok(Row {
        cols: vec![Col::Text(key), Col::Text(value)],
    })
}

#[uniffi::export]
pub fn new_log_row(
    level: String,
    category: String,
    source: String,
    session_id: String,
    message: String,
    details: Option<Vec<u8>>,
    details_type: Option<String>,
) -> Result<Row, YrsError> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    Ok(Row {
        cols: vec![
            Col::Integer(timestamp),
            Col::Text(level),
            Col::Text(category),
            Col::Text(source),
            Col::Text(session_id),
            Col::Text(message),
            details.map(Col::Blob).unwrap_or(Col::Null),
            details_type.map(Col::Text).unwrap_or(Col::Null),
        ],
    })
}

#[uniffi::export]
pub fn new_incoming_love_letter_row(
    love_letter: Vec<u8>,
    target_page_id: String,
    session_id: String,
) -> Result<Row, YrsError> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    Ok(Row {
        cols: vec![
            Col::Blob(love_letter),
            Col::Text(target_page_id),
            Col::Integer(timestamp),
            Col::Text("false".to_string()),
            Col::Text(session_id),
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::key_value_storage::key_value_storage_columns;
    use crate::tbl_backlinks::{backlinks_columns, get_foreign_def_backlinks};
    use crate::tbl_every_block_in_existence::{
        every_block_in_existence_columns, get_foreign_def_every_block_in_existence,
    };
    use crate::tbl_incoming_love_letters::incoming_love_letters_columns;
    use crate::tbl_logs::logs_columns;
    use crate::tbl_pages::pages_columns;
    use crate::tbl_uncommitted_diffs::uncommitted_diffs_columns;
    use protocol::{
        new_table::ColumnDef,
        row_col::{Col, Row},
    };

    // Helper to map a `Col` variant to the string type expected in ColumnDef
    fn col_type_string(col: &Col) -> &'static str {
        match col {
            Col::Null => "NULL",
            Col::Integer(_) => "INTEGER",
            Col::Real(_) => "REAL",
            Col::Text(_) => "TEXT",
            Col::Blob(_) => "BLOB",
        }
    }

    // Compares a Row's columns against a table's ColumnDefs, skipping auto‑increment id columns.
    fn assert_row_matches_table(row: &Row, table_def: &[ColumnDef], helper_name: &str) {
        // Remove columns that are auto‑increment primary keys (like the `id` column).
        let expected_cols: Vec<&ColumnDef> = table_def
            .iter()
            .filter(|c| !(c.primary_key && c.autoincrement))
            .collect();

        assert_eq!(
            row.cols.len(),
            expected_cols.len(),
            "{}: number of columns in row does not match table definition. \
             This test shows what {} expects; if it fails, the table definition and {} are out of sync.",
            helper_name,
            helper_name,
            helper_name
        );

        for (i, (col, def)) in row.cols.iter().zip(expected_cols.iter()).enumerate() {
            assert_eq!(
                col_type_string(col),
                def.column_type,
                "{}: column {} type mismatch (row has '{}', table expects '{}'). \
                 This test shows what {} expects; if it fails, the table definition and {} are out of sync.",
                helper_name,
                i,
                col_type_string(col),
                def.column_type,
                helper_name,
                helper_name
            );
        }
    }

    #[test]
    fn test_new_page_row_matches_table() {
        let row = new_page_row("test_page".to_string(), true, "user1".to_string()).unwrap();
        assert_row_matches_table(&row, &pages_columns(), "new_page_row");
    }

    #[test]
    fn test_new_every_block_in_existence_row_matches_table() {
        let row = new_every_block_in_existence_row(
            "Title".to_string(),
            "content".to_string(),
            "my_yrs_id".to_string(),
            "page_id".to_string(),
        )
        .unwrap();
        assert_row_matches_table(
            &row,
            &every_block_in_existence_columns(),
            "new_every_block_in_existence_row",
        );
    }

    #[test]
    fn test_new_uncommitted_diff_row_matches_table() {
        let row = new_uncommitted_diff_row(
            vec![1, 2, 3],
            vec![4, 5, 6],
            "session123".to_string(),
            "target_id".to_string(),
        )
        .unwrap();
        assert_row_matches_table(
            &row,
            &uncommitted_diffs_columns(),
            "new_uncommitted_diff_row",
        );
    }

    #[test]
    fn test_new_backlink_row_matches_table() {
        let row = new_backlink_row("owner_page".to_string(), "target_page".to_string()).unwrap();
        assert_row_matches_table(&row, &backlinks_columns(), "new_backlink_row");
    }

    #[test]
    fn test_new_key_value_item_matches_table() {
        let row = new_key_value_item("key".to_string(), "value".to_string()).unwrap();
        assert_row_matches_table(&row, &key_value_storage_columns(), "new_key_value_item");
    }

    #[test]
    fn test_new_log_row_matches_table() {
        let row = new_log_row(
            "info".to_string(),
            "sync".to_string(),
            "backend".to_string(),
            "session123".to_string(),
            "something happened".to_string(),
            Some(vec![1, 2, 3]),
            Some("SyncError".to_string()),
        )
        .unwrap();
        assert_row_matches_table(&row, &logs_columns(), "new_log_row");
    }

    #[test]
    fn test_new_incoming_love_letter_row_matches_table() {
        let row = new_incoming_love_letter_row(
            vec![1, 2, 3],
            "page_id".to_string(),
            "session123".to_string(),
        )
        .unwrap();
        assert_row_matches_table(
            &row,
            &incoming_love_letters_columns(),
            "new_incoming_love_letter_row",
        );
    }

    // Checks that every FK's `column` field actually exists in the table it claims to belong to.
    fn assert_fk_columns_exist(
        fks: &[protocol::new_table::ForeignKeyDef],
        table_def: &[ColumnDef],
        helper_name: &str,
    ) {
        for fk in fks {
            let exists = table_def.iter().any(|c| c.name == fk.column);
            assert!(
                exists,
                "{}: foreign key references column '{}' which does not exist in the table definition",
                helper_name, fk.column
            );
        }
    }

    #[test]
    fn test_backlinks_foreign_keys_valid() {
        assert_fk_columns_exist(
            &get_foreign_def_backlinks(),
            &backlinks_columns(),
            "backlinks",
        );
    }

    #[test]
    fn test_every_block_in_existence_foreign_keys_valid() {
        assert_fk_columns_exist(
            &get_foreign_def_every_block_in_existence(),
            &every_block_in_existence_columns(),
            "every_block_in_existence",
        );
    }
}
