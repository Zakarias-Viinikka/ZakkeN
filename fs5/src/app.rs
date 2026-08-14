use leptos::prelude::*;

use leptos_integration_3::local_sqlite::local_sqlite_wrapper;

#[derive(Debug, Clone)]
struct Text {
    id: usize,
    text: String,
}

#[component]
pub fn App() -> impl IntoView {
    let (texts, set_texts) = signal(Vec::new());
    let (setup_finished, finished_setup_setter) = signal(false);
    Effect::new(move || {
        //bug that's gonna create unwrap error if the table hasn't been populated and exists.
        // the effect doesn't actually check if "setup_finished" is true or not
        let table_name = "data";
        let arguments = "";
        let columns_to_read = vec![];
        let set_texts = set_texts;
        leptos::task::spawn_local(async move {
            let data =
                local_sqlite_wrapper::get_data(table_name, arguments, &columns_to_read).await;
            match data {
                Ok(data) => {
                    let texts: Vec<Text> = data
                        .into_iter() // iterate over all rows
                        .map(|row| Text {
                            id: row[0].parse().unwrap(),
                            text: row[1].to_string(),
                        })
                        .collect();

                    set_texts.set(texts);
                }
                Err(e) => eprintln!("Error: {:?}", e),
            }
            // Perform an async call or fetch
            // let result = fetch_from_api().await;
            // set_data.set(Some(result));
        });
    });
    view! {
        <crate::setup::Setup finished_setup=finished_setup_setter/>
        <div class="split">
            <div class="left">
                {move ||
                    if setup_finished.get() {
                        view! {

                        <br/>   <br/>   <br/>
                        <h2> "Default Page" </h2>
                        <For
                            // a function that returns the items we're iterating over; a signal is fine
                            each=move || texts.get()
                            // a unique key for each item
                            key=|text| text.id
                            // renders each item to a view
                            children=move |text: Text| {
                            view! {
                                "id: " {text.id}
                                <div class="happy_little_div_holding_all_the_text">
                                    {text.text}
                                </div>
                                <br/>
                            }
                            }
                        />
                        }.into_any()
                    } else {
                        view! {
                            <p> "Waiting for setup to finish..." </p>
                        }.into_any()
                    }
                }
            </div>

            <div class="right">
                <br/><br/><br/><br/><br/><br/>
                "search:"<br/>
                <input type="text" />
            </div>
        </div>
    }
}
