use client_table_blueprints::tbl_every_block_in_existence;
use error_stuff::CqrsErr;
use protocol::payload::{GetDataIn, SelectArgument, SelectArguments};
use protocol::schema_helper::DestructDbReturnCol;
use web_internal_db::db_helper;

pub async fn read_titles() -> Result<Vec<String>, CqrsErr> {
    let out = db_helper::get_data(GetDataIn {
        table_name: tbl_every_block_in_existence::TABLE_NAME.into(),
        arguments: SelectArguments::Single(SelectArgument::All),
        columns_to_read: vec![
            tbl_every_block_in_existence::IS_TITLE.name.to_string(),
            tbl_every_block_in_existence::CONTENT.name.to_string(),
        ],
    })
    .await?;

    let mut titles = Vec::new();
    for row in out.rows {
        let mut cols = row.cols.into_iter();
        let is_title = tbl_every_block_in_existence::IS_TITLE
            .try_destruct_db_col(cols.next().ok_or_else(|| {
                CqrsErr::DbErrorContainer(protocol::error::DbError::BadCode(
                    "missing is_title col".to_string(),
                ))
            })?)
            .map_err(|e| CqrsErr::DbErrorContainer(protocol::error::DbError::BadCode(e)))?
            .unwrap_or_default();
        if is_title != "true" {
            continue;
        }
        let content = tbl_every_block_in_existence::CONTENT
            .try_destruct_db_col(cols.next().ok_or_else(|| {
                CqrsErr::DbErrorContainer(protocol::error::DbError::BadCode(
                    "missing content col".to_string(),
                ))
            })?)
            .map_err(|e| CqrsErr::DbErrorContainer(protocol::error::DbError::BadCode(e)))?
            .unwrap_or_default();
        titles.push(content);
    }
    Ok(titles)
}

pub async fn read_client_state() -> Result<String, CqrsErr> {
    let user_id = crate::db::key_value_storage_db_actions::read_key_value_from_db("user_id")
        .await?
        .unwrap_or_default();
    let server_name =
        crate::db::key_value_storage_db_actions::read_key_value_from_db("server_user_name")
            .await?
            .unwrap_or_default();
    let drift =
        crate::db::key_value_storage_db_actions::read_key_value_from_db("drift_version")
            .await?
            .unwrap_or_default();
    let port = crate::owner_of_interacting_with_server::server_port::get_port()
        .map(|p| p.to_string())
        .unwrap_or_default();
    let url = web_sys::window()
        .and_then(|w| w.location().href().ok())
        .unwrap_or_default();
    let pages = db_helper::get_data(GetDataIn {
        table_name: client_table_blueprints::tbl_pages::TABLE_NAME.into(),
        arguments: SelectArguments::Single(SelectArgument::All),
        columns_to_read: vec![client_table_blueprints::tbl_pages::PAGE_ID.name.to_string()],
    })
    .await?;
    let state = leptos::serde_json::json!({
        "url": url,
        "user_id": user_id,
        "server_name": server_name,
        "drift_version": drift,
        "port": port,
        "page_count": pages.rows.len(),
    });
    Ok(state.to_string())
}
