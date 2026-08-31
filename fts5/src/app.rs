use leptos::{logging::log, prelude::*};
use leptos_router::components::A;
use protocol::{payload, serialization::Convert};

use crate::ask_js::ask;

#[derive(Debug, Clone, PartialEq)]
struct Text {
    id: usize,
    text: String,
}

async fn load_texts() -> Result<Vec<Text>, String> {
    let get_data_in = payload::GetDataIn {
        table_name: "data".to_string(),
        arguments: vec![payload::SelectArgument::All],
        columns_to_read: Vec::new(),
    };

    let bytes = ask("get_data", Some(get_data_in.to_payload()))
        .await
        .map_err(|e| format!("{e:?}"))?;

    let out = payload::GetDataOut::un_payloadify(&bytes).map_err(|e| format!("{e:?}"))?;

    Ok(out
        .rows
        .into_iter()
        .filter_map(|row| {
            let values = row.to_string_vec();

            let id = values.first()?.parse::<usize>().ok()?;
            let text = values.get(1)?.clone();

            Some(Text { id, text })
        })
        .collect())
}

/*async fn search_texts(query: String) -> Result<Vec<Text>, String> {
    let search_in = payload::SearchFts5In {
        table_name: "data".to_string(),
        text_to_lookup: query,
    };

    let response = ask("search_fts5", Some(search_in.to_payload()))
        .await
        .map_err(|e| format!("{e:?}"))?;

    let out = payload::SearchFts5Out::un_payloadify(&response).map_err(|e| format!("{e:?}"))?;

    Ok(out
        .rows
        .into_iter()
        .filter_map(|row| {
            let values = row.to_string_vec();

            let id = values.first()?.parse::<usize>().ok()?;
            let text = values.get(1)?.clone();

            Some(Text { id, text })
        })
        .collect())
}*/
async fn search_texts(query: String) -> Result<Vec<Text>, String> {
    log!("search_texts: query = {:?}", query);

    let search_in = payload::SearchFts5In {
        table_name: "data".to_string(),
        text_to_lookup: query.clone(),
    };

    let response = ask("search_fts5", Some(search_in.to_payload()))
        .await
        .map_err(|e| {
            log!("search_texts: ask failed: {:?}", e);
            format!("{e:?}")
        })?;

    log!("search_texts: response bytes length = {}", response.len());

    let out = payload::GetDataOut::un_payloadify(&response).map_err(|e| {
        log!("search_texts: un_payloadify failed: {:?}", e);
        format!("{e:?}")
    })?;

    log!("search_texts: returned {} rows", out.rows.len());

    let mut texts = Vec::new();
    for (idx, row) in out.rows.into_iter().enumerate() {
        let values = row.to_string_vec();
        log!("search_texts: row {} -> {:?}", idx, values);

        let id = match values.first() {
            Some(id_str) => match id_str.parse::<usize>() {
                Ok(id) => id,
                Err(e) => {
                    log!(
                        "search_texts: row {} parse id failed: {:?} (id_str={:?})",
                        idx,
                        e,
                        id_str
                    );
                    continue;
                }
            },
            None => {
                log!("search_texts: row {} missing id", idx);
                continue;
            }
        };

        let text = match values.get(1) {
            Some(t) => t.clone(),
            None => {
                log!("search_texts: row {} missing text", idx);
                continue;
            }
        };

        texts.push(Text { id, text });
    }

    log!("search_texts: successfully parsed {} texts", texts.len());
    Ok(texts)
}

#[component]
pub fn App() -> impl IntoView {
    let (texts, set_texts) = signal(Vec::<Text>::new());
    let (setup_finished, set_setup_finished) = signal(false);

    let (search_query, set_search_query) = signal(String::new());
    let (search_results, set_search_results) = signal(Vec::<Text>::new());

    // Initial data load.
    //
    // This Effect reads no reactive values, so it runs once.
    Effect::new(move || {
        leptos::task::spawn_local(async move {
            match load_texts().await {
                Ok(new_texts) => set_texts.set(new_texts),
                Err(error) => eprintln!("Failed to load data: {error}"),
            }
        });
    });

    // Search effect — runs when search_query changes.
    Effect::new(move || {
        let query = search_query.get();

        if query.trim().is_empty() {
            set_search_results.set(Vec::new());
            return;
        }

        let set_search_results = set_search_results;
        leptos::task::spawn_local(async move {
            match search_texts(query).await {
                Ok(results) => set_search_results.set(results),
                Err(error) => eprintln!("Search failed: {error}"),
            }
        });
    });

    view! {
        <crate::setup::Setup finished_setup=set_setup_finished/>
        <A href="/dbgui">"Db Gui"</A>
        <div class="split">
            <div class="left">
                {move || {
                    if setup_finished.get() {
                        view! {
                            <br/>
                            <br/>
                            <br/>

                            <h2>"Default Page"</h2>

                            <For
                                each=move || texts.get()
                                key=|text| text.id
                                children=move |text| {
                                    view! {
                                        "id: " {text.id}

                                        <div class="happy_little_div_holding_all_the_text">
                                            {text.text}
                                        </div>

                                        <br/>
                                    }
                                }
                            />
                        }
                        .into_any()
                    } else {
                        view! {
                            <p>"Waiting for setup to finish..."</p>
                        }
                        .into_any()
                    }
                }}
            </div>

            <div class="right">
                <br/>
                <br/>
                <br/>
                <br/>
                <br/>
                <br/>

                "search:"
                <br/>

                <input
                    type="text"
                    on:input:target=move |ev| {
                        set_search_query.set(ev.target().value());
                    }
                    prop:value=move || search_query.get()
                />

                <br/>
                <br/>

                <span>"search result goes here:"</span>
                <br/>

                <For
                    each=move || search_results.get()
                    key=|text| text.id
                    children=move |text| {
                        view! {
                            "id: " {text.id}
                            <div>{text.text}</div>
                            <br/>
                        }
                    }
                />
            </div>
        </div>
    }
}
