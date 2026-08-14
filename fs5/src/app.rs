use leptos::{attr::Columnlines, prelude::*, reactive::effect, task::spawn_local};

use leptos_integration_3::local_sqlite::local_sqlite_wrapper;

struct Text {
    id: usize,
    text: String,
}

#[component]
pub fn App() -> impl IntoView {
    let (texts, set_texts) = signal(Vec::new());
    Effect::new(|| {
        let table_name = "data";
        let arguments = "";
        let columns_to_read = vec!["".to_string()];
        let set_texts = set_texts;
        leptos::task::spawn_local(async move {
            let data =
                local_sqlite_wrapper::get_data(table_name, arguments, &columns_to_read).await;
            match data {
                Ok(data) => {
                    if let Some(data) = data.into_iter().next() {
                        let texts = data
                            .into_iter()
                            .enumerate()
                            .map(|(index, text)| Text {
                                id: index,
                                text: text,
                            })
                            .collect();
                        set_texts.set(texts);
                    }
                }
                Err(e) => eprintln!("Error: {}", e),
            }
            // Perform an async call or fetch
            // let result = fetch_from_api().await;
            // set_data.set(Some(result));
        });
    });
    view! {
        <Setup />
        <br/>   <br/>   <br/>
        <h2> "Default Page" </h2>
        <For
            // a function that returns the items we're iterating over; a signal is fine
            each=move || texts.get()
            // a unique key for each item
            key=|text| text.id
            // renders each item to a view
            children=move |counter: Counter| {
            view! {
                <button>"Value: " {move || counter.count.get()}</button>
            }
            }
        />
    }
}
