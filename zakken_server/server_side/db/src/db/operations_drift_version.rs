use client_table_blueprints::tbl_key_value_storage;
use db_wrapper::android_mascot::LiveForever;
use protocol::error::DbError;
use protocol::payload::{
    ColumnValue, CreateTableIn, DeleteRowWhereIn, GetDataIn, InsertDataIn, SelectArgument,
    SelectArguments,
};
use protocol::row_col::Col;
use protocol::schema_helper::DestructDbReturnCol;

use crate::db::operations_key_value;
use crate::db::schema::tbl_drift_version;

const DRIFT_VERSION_ID: &str = "drift_version_id";

#[derive(Clone, Debug)]
pub struct DriftVersion {
    pub version: String,
    pub reason: String,
}

pub fn create_table_if_not_exist(liver: &LiveForever) -> Result<(), DbError> {
    let list = liver.list_tables()?;
    if !list
        .table_names
        .iter()
        .any(|t| t == tbl_drift_version::TABLE_NAME)
    {
        liver.create_table(CreateTableIn {
            table_name: tbl_drift_version::TABLE_NAME.to_string(),
            columns: tbl_drift_version::new_table_drift_version(),
        })?;
    }
    Ok(())
}

/// Read the current (version, reason) pair, or None if none has been minted.
pub fn get_current(liver: &LiveForever) -> Result<Option<DriftVersion>, DbError> {
    let Some(id_str) = operations_key_value::get_value(liver, DRIFT_VERSION_ID)? else {
        return Ok(None);
    };
    let out = liver.get_data(GetDataIn {
        table_name: tbl_drift_version::TABLE_NAME.to_string(),
        arguments: SelectArguments::Single(SelectArgument::XEqualY {
            x: "id".to_string(),
            y: id_str,
        }),
        columns_to_read: vec![
            tbl_drift_version::VERSION.name.to_string(),
            tbl_drift_version::REASON.name.to_string(),
        ],
    })?;
    let Some(row) = out.rows.into_iter().next() else {
        return Ok(None);
    };
    let mut cols = row.cols.into_iter();
    let version = tbl_drift_version::VERSION
        .try_destruct_db_col(cols.next().ok_or_else(|| {
            DbError::BadCode("drift_version row missing version col".to_string())
        })?)
        .map_err(DbError::BadCode)?
        .ok_or_else(|| DbError::BadCode("drift_version version was null".to_string()))?;
    let reason = tbl_drift_version::REASON
        .try_destruct_db_col(cols.next().ok_or_else(|| {
            DbError::BadCode("drift_version row missing reason col".to_string())
        })?)
        .map_err(DbError::BadCode)?
        .ok_or_else(|| DbError::BadCode("drift_version reason was null".to_string()))?;
    Ok(Some(DriftVersion { version, reason }))
}

/// Mint a new drift version row and point key_value_storage at it.
/// Returns (version, reason). Old rows are kept as history.
pub fn bump(liver: &LiveForever, reason: &str) -> Result<DriftVersion, DbError> {
    let version = new_server_drift_version();
    let row = vec![
        ColumnValue {
            column_name: tbl_drift_version::VERSION.name.to_string(),
            value: Col::Text(version.clone()),
        },
        ColumnValue {
            column_name: tbl_drift_version::REASON.name.to_string(),
            value: Col::Text(reason.to_string()),
        },
    ];
    liver.insert_data(InsertDataIn {
        table_name: tbl_drift_version::TABLE_NAME.to_string(),
        values: row,
    })?;

    let lookup = liver.get_data(GetDataIn {
        table_name: tbl_drift_version::TABLE_NAME.to_string(),
        arguments: SelectArguments::Single(SelectArgument::XEqualY {
            x: tbl_drift_version::VERSION.name.to_string(),
            y: version.clone(),
        }),
        columns_to_read: vec!["id".to_string()],
    })?;
    let row = lookup
        .rows
        .into_iter()
        .next()
        .ok_or_else(|| DbError::BadCode("bump: inserted row not found".to_string()))?;
    let id = row
        .cols
        .into_iter()
        .next()
        .ok_or_else(|| DbError::BadCode("bump: row missing id".to_string()))?;
    let id_str = match id {
        Col::Integer(i) => i.to_string(),
        other => {
            return Err(DbError::BadCode(format!(
                "bump: expected Integer id, got {other:?}"
            )));
        }
    };

    liver.delete_row_where(DeleteRowWhereIn {
        table_name: tbl_key_value_storage::TABLE_NAME.to_string(),
        arguments: SelectArguments::Single(SelectArgument::XEqualY {
            x: tbl_key_value_storage::KEY.name.to_string(),
            y: DRIFT_VERSION_ID.to_string(),
        }),
    })?;
    let kv_row = tbl_key_value_storage::new_key_value_item(
        DRIFT_VERSION_ID.to_string(),
        id_str,
    )
    .map_err(|e| DbError::BadCode(format!("kv row for drift_version_id: {e:?}")))?;
    liver.insert_data(InsertDataIn {
        table_name: tbl_key_value_storage::TABLE_NAME.to_string(),
        values: kv_row,
    })?;

    Ok(DriftVersion {
        version,
        reason: reason.to_string(),
    })
}

/// Mint at startup if no drift version exists yet. Returns the current one.
pub fn ensure_minted(liver: &LiveForever) -> Result<DriftVersion, DbError> {
    if let Some(curr) = get_current(liver)? {
        return Ok(curr);
    }
    bump(liver, "")
}

fn new_server_drift_version() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("server-{}", now.as_micros())
}
