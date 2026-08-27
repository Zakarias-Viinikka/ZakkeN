use anyhow::Result;
use leptos::logging::log;
use leptos::prelude::*;
use leptos::task::spawn_local;
//use leptos_integration_3::local_sqlite::column_helper;
//use leptos_integration_3::local_sqlite::local_sqlite_wrapper;
use crate::ask_js::ask;
use protocol::{payload, row_col, serialization::Convert};
use random_word::{self, Lang};

const TABLE_NAME: &str = "data";

#[component]
pub fn Setup(finished_setup: WriteSignal<bool>) -> impl IntoView {
    Effect::new(move || {
        let finished_setup = finished_setup.clone();
        spawn_local(async move {
            let create_table_result = create_table_if_not_exist().await;
            create_table_result
                .err()
                .map(|e| log!("create_table_if_not_exist error: {}", e));
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

    let columns = data_col_def();
    let create_table_payload = payload::CreateTableIn {
        table_name: TABLE_NAME.into(),
        columns: columns,
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
        table_name: "data".to_string(),
        arguments: vec![payload::SelectArgument::All],
        columns_to_read: vec!["".to_string()], // empty string to get all columns
    }
    .to_payload();

    let response = ask("get_data", Some(get_data_payload))
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    let get_data_out = payload::GetDataOut::un_payloadify(&response)?;

    if get_data_out.rows.is_empty() {
        for _ in 0..100 {
            let insert_payload = payload::InsertDataIn {
                table_name: "data".to_string(),
                values: vec![payload::ColumnValue {
                    column_name: "random_words".to_string(),
                    value: row_col::Col::Text(generate_random_words()), // adjust variant if needed
                }],
            }
            .to_payload();

            ask("insert_data", Some(insert_payload))
                .await
                .map_err(|e| anyhow::anyhow!(e))?;
        }
        log!("inserted 100 random word rows");
        Ok(())
    } else {
        log!("columns exist in db already");
        Ok(())
    }
}

fn generate_random_words() -> String {
    let mut words = Vec::new();
    for _ in 0..5 {
        words.push(random_word::get(Lang::En));
    }
    words.join(" ")
}

use protocol::new_table;
fn data_col_def() -> Vec<new_table::ColumnDef> {
    vec![
        new_table::id_column(),
        new_table::default_col(new_table::ColumnType::Text, "random_words"),
    ]
}
