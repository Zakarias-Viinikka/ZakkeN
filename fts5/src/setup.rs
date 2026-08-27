use anyhow::{Result, anyhow};
use leptos::logging::log;
use leptos::prelude::*;
use leptos::task::spawn_local;
//use leptos_integration_3::local_sqlite::column_helper;
//use leptos_integration_3::local_sqlite::local_sqlite_wrapper;
use random_word::{self, Lang};

#[component]
pub fn Setup(finished_setup: WriteSignal<bool>) -> impl IntoView {
    Effect::new(move || {
        let finished_setup = finished_setup.clone();
        spawn_local(async move {
            create_table_if_not_exist().await;
            finished_setup.set(true);
        });
    });
}

pub async fn create_table_if_not_exist() -> Result<()> {
    let tables = local_sqlite_wrapper::list_tables()
        .await
        .map_err(|e| anyhow!("{:?}", e))?;
    if tables.iter().any(|t| t == "data") {
        log!("data table already exists");
        return Ok(());
    }

    let columns = data_col_def();

    let msg = local_sqlite_wrapper::create_table("data", &columns)
        .await
        .map_err(|e| anyhow!("{:?}", e))?;
    log!("{}", msg);
    create_hardcoded_columns_if_not_exist().await?;
    Ok(())
}

pub async fn create_hardcoded_columns_if_not_exist() -> Result<()> {
    let table_name = "data";
    let arguments = "";
    let columns_to_read = vec!["".to_string()]; //returns all of them
    let result = local_sqlite_wrapper::get_data(table_name, arguments, &columns_to_read).await;
    match result {
        Ok(result) => {
            if result.into_iter().next().is_none() {
                for _ in 0..100 {
                    let column_names = vec!["random_words".to_string()];
                    let column_values = vec![generate_random_words()];
                    local_sqlite_wrapper::insert_data(table_name, &column_names, &column_values)
                        .await
                        .map_err(|e| anyhow!(format!("{:?}", e)))?;
                }
                log!("created 5 columns");
                Ok(())
            } else {
                log!("columns exist in db already");
                Ok(())
            }
        }
        Err(e) => Err(anyhow!(format!("{:?}", e))),
    }
}

fn generate_random_words() -> String {
    let mut words = Vec::new();
    for _ in 0..5 {
        words.push(random_word::get(Lang::En));
    }
    words.join(" ")
}

fn data_col_def() -> Vec<local_sqlite_wrapper::CreateTableColumnDef> {
    vec![
        column_helper::id_column(),
        column_helper::column("random_words", "TEXT"),
    ]
}
