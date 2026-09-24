use my_yrs_lib::yrs_backlinks::YrsBacklinks;
use my_yrs_lib::{YrsActivePages, YrsError};
use protocol::payload::ColumnValue;
use protocol::row_col::Col;
use std::sync::Arc;

#[uniffi::export]
pub fn new_page_row(
    page_id: String,
    is_main_menu_page: bool,
    yrs_page_blobbed: Vec<u8>,
    version: Vec<u8>, //version is the yrs representation of "what version is this doc"
) -> Result<Vec<ColumnValue>, YrsError> {
    let active_doc = Arc::new(YrsActivePages::new_empty());
    let page_status = active_doc.snapshot()?;

    Ok(vec![
        ColumnValue {
            column_name: "page_id".to_string(),
            value: Col::Text(page_id),
        },
        ColumnValue {
            column_name: "blobbed_page".to_string(),
            value: Col::Blob(yrs_page_blobbed),
        },
        ColumnValue {
            column_name: "page_status".to_string(),
            value: Col::Blob(page_status),
        },
        ColumnValue {
            column_name: "version".to_string(),
            value: Col::Blob(version),
        },
        ColumnValue {
            column_name: "is_main_menu_page".to_string(),
            value: Col::Text(if is_main_menu_page {
                "true".to_string()
            } else {
                "false".to_string()
            }),
        },
    ])
}

#[uniffi::export]
pub fn new_every_block_in_existence_row(
    is_title: bool,
    content: String,
    my_id_as_given_by_yrs: String,
    id_of_page_i_belong_to: String,
    position: f64,
) -> Result<Vec<ColumnValue>, YrsError> {
    Ok(vec![
        ColumnValue {
            column_name: "is_title".to_string(),
            value: Col::Text(if is_title {
                "true".to_string()
            } else {
                "false".to_string()
            }),
        },
        ColumnValue {
            column_name: "content".to_string(),
            value: Col::Text(content),
        },
        ColumnValue {
            column_name: "my_id_as_given_by_yrs".to_string(),
            value: Col::Text(my_id_as_given_by_yrs),
        },
        ColumnValue {
            column_name: "id_of_page_i_belong_to".to_string(),
            value: Col::Text(id_of_page_i_belong_to),
        },
        ColumnValue {
            column_name: "position".to_string(),
            value: Col::Real(position),
        },
    ])
}

#[uniffi::export]
pub fn new_uncommitted_diff_row(
    snapshot_of_edit: Vec<u8>,
    love_letter_sketch: Vec<u8>,
    session_id: String,
    target_id: String,
) -> Result<Vec<ColumnValue>, YrsError> {
    Ok(vec![
        ColumnValue {
            column_name: "snapshot_of_edit".to_string(),
            value: Col::Blob(snapshot_of_edit),
        },
        ColumnValue {
            column_name: "love_letter_sketch".to_string(),
            value: Col::Blob(love_letter_sketch),
        },
        ColumnValue {
            column_name: "session_id".to_string(),
            value: Col::Text(session_id),
        },
        ColumnValue {
            column_name: "target_id".to_string(),
            value: Col::Text(target_id),
        },
    ])
}

#[uniffi::export]
pub fn new_backlink_row(
    page_that_holds_link_id: String,
    page_being_linked_to_id: String,
) -> Result<Vec<ColumnValue>, YrsError> {
    let backlinks_doc = Arc::new(YrsBacklinks::new_empty());
    let disabled = backlinks_doc.clone().snapshot()?;
    let version = backlinks_doc.create_bookmark_of_synced_state()?;

    Ok(vec![
        ColumnValue {
            column_name: "page_that_holds_link_id".to_string(),
            value: Col::Text(page_that_holds_link_id),
        },
        ColumnValue {
            column_name: "page_being_linked_to_id".to_string(),
            value: Col::Text(page_being_linked_to_id),
        },
        ColumnValue {
            column_name: "disabled".to_string(),
            value: Col::Blob(disabled),
        },
        ColumnValue {
            column_name: "version".to_string(),
            value: Col::Blob(version),
        },
    ])
}

#[uniffi::export]
pub fn new_key_value_item(key: String, value: String) -> Result<Vec<ColumnValue>, YrsError> {
    Ok(vec![
        ColumnValue {
            column_name: "key".to_string(),
            value: Col::Text(key),
        },
        ColumnValue {
            column_name: "value".to_string(),
            value: Col::Text(value),
        },
    ])
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
) -> Result<Vec<ColumnValue>, YrsError> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    Ok(vec![
        ColumnValue {
            column_name: "timestamp".to_string(),
            value: Col::Integer(timestamp),
        },
        ColumnValue {
            column_name: "level".to_string(),
            value: Col::Text(level),
        },
        ColumnValue {
            column_name: "category".to_string(),
            value: Col::Text(category),
        },
        ColumnValue {
            column_name: "source".to_string(),
            value: Col::Text(source),
        },
        ColumnValue {
            column_name: "session_id".to_string(),
            value: Col::Text(session_id),
        },
        ColumnValue {
            column_name: "message".to_string(),
            value: Col::Text(message),
        },
        ColumnValue {
            column_name: "details".to_string(),
            value: details.map(Col::Blob).unwrap_or(Col::Null),
        },
        ColumnValue {
            column_name: "details_type".to_string(),
            value: details_type.map(Col::Text).unwrap_or(Col::Null),
        },
    ])
}

#[uniffi::export]
pub fn new_incoming_love_letter_row(
    love_letter: Vec<u8>,
    target_page_id: String,
    session_id: String,
) -> Result<Vec<ColumnValue>, YrsError> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    Ok(vec![
        ColumnValue {
            column_name: "love_letter".to_string(),
            value: Col::Blob(love_letter),
        },
        ColumnValue {
            column_name: "target_page_id".to_string(),
            value: Col::Text(target_page_id),
        },
        ColumnValue {
            column_name: "timestamp".to_string(),
            value: Col::Integer(timestamp),
        },
        ColumnValue {
            column_name: "applied".to_string(),
            value: Col::Text("false".to_string()),
        },
        ColumnValue {
            column_name: "session_id".to_string(),
            value: Col::Text(session_id),
        },
    ])
}

//IMPORTANT
// IMPORTANT
// IMPORTANT
//
// NEED TO REWRITE ALL OF THESE BY HAND AT SOME POINT
//
// FAKE TESTS ALERT
// FAKE TESTS ALERT
//
// maybe they're not fake. but i have 0 verifcation for that.
// don't remove this comment until tests have been verified or rewritten

#[cfg(test)]
mod tests {
    //use super::*;
    //use crate::key_value_storage::key_value_storage_columns;
    use crate::tbl_backlinks::{backlinks_columns, get_foreign_def_backlinks};
    use crate::tbl_every_block_in_existence::{
        every_block_in_existence_columns, get_foreign_def_every_block_in_existence,
    };
    //use crate::tbl_incoming_love_letters::incoming_love_letters_columns;
    //use crate::tbl_logs::logs_columns;
    //use crate::tbl_pages::pages_columns;
    //use crate::tbl_uncommitted_diffs::uncommitted_diffs_columns;
    use protocol::{
        new_table::ColumnDef,
        //row_col::{Col, Row},
    };

    // Helper to map a `Col` variant to the string type expected in ColumnDef
    /*fn col_type_string(col: &Col) -> &'static str {
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
    }*/

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
