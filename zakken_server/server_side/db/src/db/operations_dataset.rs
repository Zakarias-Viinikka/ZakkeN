use crate::db::schema::tbl_love_letters_from_clients;
use crate::db::schema::tbl_registered_users;
use crate::db::schema::tbl_unsent_love_letters;
use client_table_blueprints::{tbl_key_value_storage, tbl_logs, tbl_pages};
use db_wrapper::android_mascot::LiveForever;
use protocol::error::DbError;
use protocol::payload::DeleteAllRowsIn;

pub fn wipe_dataset_tables(liver: &LiveForever) -> Result<(), DbError> {
    liver.delete_all_rows(DeleteAllRowsIn {
        table_name: tbl_unsent_love_letters::TABLE_NAME.to_string(),
    })?;
    liver.delete_all_rows(DeleteAllRowsIn {
        table_name: tbl_love_letters_from_clients::TABLE_NAME.to_string(),
    })?;
    liver.delete_all_rows(DeleteAllRowsIn {
        table_name: tbl_pages::TABLE_NAME.to_string(),
    })?;
    liver.delete_all_rows(DeleteAllRowsIn {
        table_name: tbl_registered_users::TABLE_NAME.to_string(),
    })?;
    liver.delete_all_rows(DeleteAllRowsIn {
        table_name: tbl_logs::TABLE_NAME.to_string(),
    })?;
    liver.delete_all_rows(DeleteAllRowsIn {
        table_name: tbl_key_value_storage::TABLE_NAME.to_string(),
    })?;
    Ok(())
}
