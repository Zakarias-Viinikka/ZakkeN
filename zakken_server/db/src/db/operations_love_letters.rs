use crate::db::schema::tbl_love_letters_from_clients;
use crate::db::schema::tbl_registered_users;
use crate::db::schema::tbl_unsent_love_letters;
use db_wrapper::android_mascot::LiveForever;
use protocol::error::DbError;
use protocol::payload::{
    ColumnValue, CountRowsIn, CreateTableIn, DeleteRowWhereIn, GetDataIn, GetSingleColIn,
    InsertDataAndGetColIn, InsertDataIn, JoinType, SelectArgument, SelectArguments,
};
use protocol::row_col::Col;
use protocol::schema_helper::DestructDbReturnCol;

pub fn create_tables_if_not_exist(liver: &LiveForever) -> Result<(), DbError> {
    let list = liver.list_tables()?;

    if !list
        .table_names
        .iter()
        .any(|t| t == tbl_love_letters_from_clients::TABLE_NAME)
    {
        liver.create_table(CreateTableIn {
            table_name: tbl_love_letters_from_clients::TABLE_NAME.to_string(),
            columns: tbl_love_letters_from_clients::new_table_love_letters_from_clients(),
        })?;
    }

    if !list
        .table_names
        .iter()
        .any(|t| t == tbl_unsent_love_letters::TABLE_NAME)
    {
        liver.create_table(CreateTableIn {
            table_name: tbl_unsent_love_letters::TABLE_NAME.to_string(),
            columns: tbl_unsent_love_letters::new_table_unsent_love_letters(),
        })?;
    }

    Ok(())
}

pub fn insert_letter(
    liver: &LiveForever,
    love_letter: Vec<u8>,
    target_page: String,
) -> Result<i64, DbError> {
    let out = liver.insert_data_and_get_col(InsertDataAndGetColIn {
        table_name: tbl_love_letters_from_clients::TABLE_NAME.to_string(),
        values: vec![
            ColumnValue {
                column_name: tbl_love_letters_from_clients::LOVE_LETTER
                    .name
                    .to_string(),
                value: Col::Blob(love_letter),
            },
            ColumnValue {
                column_name: tbl_love_letters_from_clients::TARGET_PAGE
                    .name
                    .to_string(),
                value: Col::Text(target_page),
            },
        ],
        column_to_return: "id".to_string(),
    })?;

    let id = out
        .value
        .as_int()
        .map_err(|e| DbError::BadCode(format!("letter id was not an int: {}", e)))?;

    Ok(*id)
}

pub fn get_letter(liver: &LiveForever, letter_id: i64) -> Result<Vec<u8>, DbError> {
    let out = liver.get_single_col(GetSingleColIn {
        table_name: tbl_love_letters_from_clients::TABLE_NAME.to_string(),
        arguments: SelectArguments::Single(SelectArgument::XEqualY {
            x: "id".to_string(),
            y: letter_id.to_string(),
        }),
        column_to_read: tbl_love_letters_from_clients::LOVE_LETTER
            .name
            .to_string(),
    })?;

    out.value
        .as_blob()
        .map(|b| b.to_vec())
        .map_err(|e| DbError::BadCode(format!("letter blob read failed: {}", e)))
}

pub fn get_all_registered_user_names(liver: &LiveForever) -> Result<Vec<String>, DbError> {
    let out = liver.get_data(GetDataIn {
        table_name: tbl_registered_users::TABLE_NAME.to_string(),
        arguments: SelectArguments::Single(SelectArgument::All),
        columns_to_read: vec![tbl_registered_users::USER_NAME.name.to_string()],
    })?;

    let mut names = Vec::new();
    for row in out.rows {
        if let Some(col) = row.cols.into_iter().next() {
            let name = tbl_registered_users::USER_NAME
                .try_destruct_db_col(col)
                .map_err(DbError::BadCode)?
                .ok_or_else(|| DbError::BadCode("user_name was null".to_string()))?;
            names.push(name);
        }
    }
    Ok(names)
}

pub fn get_unsent_letter_ids_for_user(
    liver: &LiveForever,
    user_name: &str,
) -> Result<Vec<i64>, DbError> {
    let out = liver.get_data(GetDataIn {
        table_name: tbl_unsent_love_letters::TABLE_NAME.to_string(),
        arguments: SelectArguments::Single(SelectArgument::XEqualY {
            x: tbl_unsent_love_letters::USER_NAME.name.to_string(),
            y: user_name.to_string(),
        }),
        columns_to_read: vec![tbl_unsent_love_letters::LETTER_ID.name.to_string()],
    })?;

    let mut ids = Vec::new();
    for row in out.rows {
        if let Some(col) = row.cols.into_iter().next() {
            let id = tbl_unsent_love_letters::LETTER_ID
                .try_destruct_db_col(col)
                .map_err(DbError::BadCode)?
                .ok_or_else(|| DbError::BadCode("letter_id was null".to_string()))?;
            ids.push(id);
        }
    }
    Ok(ids)
}

pub fn user_has_unsent_row(
    liver: &LiveForever,
    user_name: &str,
    letter_id: i64,
) -> Result<bool, DbError> {
    let out = liver.count_rows(CountRowsIn {
        table_name: tbl_unsent_love_letters::TABLE_NAME.to_string(),
        arguments: SelectArguments::Two {
            first: SelectArgument::XEqualY {
                x: tbl_unsent_love_letters::USER_NAME.name.to_string(),
                y: user_name.to_string(),
            },
            join: JoinType::And,
            second: SelectArgument::XEqualY {
                x: tbl_unsent_love_letters::LETTER_ID.name.to_string(),
                y: letter_id.to_string(),
            },
        },
    })?;
    Ok(out.count > 0)
}

pub fn queue_unsent_for_user(
    liver: &LiveForever,
    user_name: &str,
    letter_id: i64,
) -> Result<(), DbError> {
    if user_has_unsent_row(liver, user_name, letter_id)? {
        return Ok(());
    }

    liver.insert_data(InsertDataIn {
        table_name: tbl_unsent_love_letters::TABLE_NAME.to_string(),
        values: vec![
            ColumnValue {
                column_name: tbl_unsent_love_letters::USER_NAME.name.to_string(),
                value: Col::Text(user_name.to_string()),
            },
            ColumnValue {
                column_name: tbl_unsent_love_letters::LETTER_ID.name.to_string(),
                value: Col::Integer(letter_id),
            },
        ],
    })
}

pub fn delete_unsent_row(
    liver: &LiveForever,
    user_name: &str,
    letter_id: i64,
) -> Result<(), DbError> {
    liver.delete_row_where(DeleteRowWhereIn {
        table_name: tbl_unsent_love_letters::TABLE_NAME.to_string(),
        arguments: SelectArguments::Two {
            first: SelectArgument::XEqualY {
                x: tbl_unsent_love_letters::USER_NAME.name.to_string(),
                y: user_name.to_string(),
            },
            join: JoinType::And,
            second: SelectArgument::XEqualY {
                x: tbl_unsent_love_letters::LETTER_ID.name.to_string(),
                y: letter_id.to_string(),
            },
        },
    })
}