use protocol::new_table::{ColumnDef, ColumnType, id_column, not_null_col, not_null_unique_col};

use crate::web_migrations::helpers::ADbTable;

pub fn entire_table() -> Vec<ADbTable> {
    vec![
        ADbTable {
            table_name: "client_love_letter".to_string(),
            columns: client_love_letter_columns(),
        },
        ADbTable {
            table_name: "every_block_in_existence".to_string(),
            columns: every_block_in_existence_columns(),
        },
        ADbTable {
            table_name: "key_value_storage".to_string(),
            columns: key_value_storage_columns(),
        },
        ADbTable {
            table_name: "pages".to_string(),
            columns: pages_columns(),
        },
        ADbTable {
            table_name: "seen_server_letters".to_string(),
            columns: seen_server_letters_columns(),
        },
    ]
}

pub fn client_love_letter_columns() -> Vec<ColumnDef> {
    vec![
        id_column(),
        not_null_col(ColumnType::Blob, "snapshot_of_edit"),
        not_null_col(ColumnType::Blob, "love_letter_sketch"),
        not_null_col(ColumnType::Text, "session_id"),
        not_null_col(ColumnType::Text, "target_id"),
    ]
}

pub fn every_block_in_existence_columns() -> Vec<ColumnDef> {
    vec![
        id_column(),
        not_null_col(ColumnType::Text, "is_title"),
        not_null_col(ColumnType::Text, "is_part_of_main_menu_page"),
        not_null_col(ColumnType::Text, "content"),
        not_null_col(ColumnType::Text, "my_id_as_given_by_yrs"),
        not_null_col(ColumnType::Text, "id_of_page_i_belong_to"),
        not_null_col(ColumnType::Text, "frac_position"),
        not_null_col(ColumnType::Text, "page_is_disabled"),
    ]
}

pub fn key_value_storage_columns() -> Vec<ColumnDef> {
    vec![
        id_column(),
        not_null_unique_col(ColumnType::Text, "key"),
        not_null_col(ColumnType::Text, "value"),
    ]
}

pub fn pages_columns() -> Vec<ColumnDef> {
    vec![
        id_column(),
        not_null_unique_col(ColumnType::Text, "page_id"),
        not_null_col(ColumnType::Blob, "blobbed_page"),
        not_null_col(ColumnType::Blob, "page_status"),
        not_null_col(ColumnType::Blob, "version"),
        not_null_col(ColumnType::Text, "is_main_menu_page"),
    ]
}

pub fn seen_server_letters_columns() -> Vec<ColumnDef> {
    vec![
        id_column(),
        not_null_col(ColumnType::Integer, "letter_id"),
    ]
}
