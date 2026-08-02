use leptos::prelude::*;

#[component]
pub fn SwapSection() -> impl IntoView {
    view! {
        <section>
            <h2>"swap"</h2>
            <label for="swap-row-id-1">"row id 1"</label>
            <input id="swap-row-id-1" type="text" />
            <label for="swap-row-id-2">"row id 2"</label>
            <input id="swap-row-id-2" type="text" />
            <label for="swap-col">"column name"</label>
            <input id="swap-col" type="text" />
            <button>"Swap columns"</button>
        </section>
    }
}
