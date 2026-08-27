use anyhow::Result;
use leptos::logging::log;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::ask_js::ask;
use protocol::new_table;
use protocol::{payload, row_col, serialization::Convert};
use random_word::{self, Lang};

const TABLE_NAME: &str = "data";

#[component]
pub fn Setup(finished_setup: WriteSignal<bool>) -> impl IntoView {
    Effect::new(move || {
        spawn_local(async move {
            if let Err(e) = create_table_if_not_exist().await {
                log!("create_table_if_not_exist error: {}", e);
                return;
            }

            if let Err(e) = create_fts5_if_not_exist().await {
                log!("create_fts5_if_not_exist error: {}", e);
                return;
            }

            if let Err(e) = rebuild_fts5_index().await {
                log!("rebuild_fts5_index error: {}", e);
                return;
            }

            finished_setup.set(true);
        });
    });
}

pub async fn create_table_if_not_exist() -> Result<()> {
    let response = ask("list_tables", None)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    let tables = payload::ListTablesOut::un_payloadify(&response)?;

    if tables.table_names.iter().any(|t| t == TABLE_NAME) {
        log!("data table already exists");
        return Ok(());
    }

    let create_table_payload = payload::CreateTableIn {
        table_name: TABLE_NAME.into(),
        columns: table_columns(),
    }
    .to_payload();

    ask("create_table", Some(create_table_payload))
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    create_hardcoded_columns_if_not_exist().await?;

    Ok(())
}

pub async fn create_hardcoded_columns_if_not_exist() -> Result<()> {
    let get_data_payload = payload::GetDataIn {
        table_name: TABLE_NAME.to_string(),
        arguments: vec![payload::SelectArgument::All],
        columns_to_read: vec!["".to_string()],
    }
    .to_payload();

    let response = ask("get_data", Some(get_data_payload))
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    let get_data_out = payload::GetDataOut::un_payloadify(&response)?;

    if get_data_out.rows.is_empty() {
        for _ in 0..100 {
            let insert_payload = payload::InsertDataIn {
                table_name: TABLE_NAME.to_string(),
                values: vec![payload::ColumnValue {
                    column_name: "random_words".to_string(),
                    value: row_col::Col::Text(generate_random_words()),
                }],
            }
            .to_payload();

            ask("insert_data", Some(insert_payload))
                .await
                .map_err(|e| anyhow::anyhow!(e))?;
        }

        log!("inserted 100 random word rows");
    } else {
        log!("columns exist in db already");
    }

    Ok(())
}

fn generate_random_words() -> String {
    let mut words = Vec::new();

    for _ in 0..5 {
        words.push(random_word::get(Lang::En));
    }

    words.join(" ")
}

fn table_columns() -> Vec<new_table::ColumnDef> {
    vec![
        new_table::id_column(),
        new_table::default_col(new_table::ColumnType::Text, "random_words"),
    ]
}

async fn create_fts5_if_not_exist() -> Result<()> {
    if check_if_fts5_exists().await? {
        log!("fts5 table exists");
        return Ok(());
    }

    let create_fts5_table_in = payload::CreateFts5TableIn {
        source_table_name: TABLE_NAME.into(),
        columns: vec!["random_words".into()],
    }
    .to_payload();

    ask("create_fts5_table", Some(create_fts5_table_in))
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    log!("created fts5 table");

    Ok(())
}

async fn rebuild_fts5_index() -> Result<()> {
    let rebuild_fts5_in = payload::RebuildFts5In {
        table_name: TABLE_NAME.into(),
    }
    .to_payload();

    ask("rebuild_fts5_index", Some(rebuild_fts5_in))
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    log!("rebuilt fts5 index");

    Ok(())
}

async fn check_if_fts5_exists() -> Result<bool> {
    const EXPECTED_FTS5_TABLE: &str = "fts5_data";

    let response = ask("list_tables", None)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    let tables = payload::ListTablesOut::un_payloadify(&response)?;

    Ok(tables.table_names.iter().any(|t| t == EXPECTED_FTS5_TABLE))
}
