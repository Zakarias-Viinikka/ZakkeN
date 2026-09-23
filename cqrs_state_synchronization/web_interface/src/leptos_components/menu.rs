use leptos::prelude::*;
use leptos_router::components::A;

struct PagesToNavTo {
    id: usize,
    title: String,
}

#[component]
pub fn Menu() -> impl IntoView {
    let (list, list_set) = signal(Vec::<PagesToNavTo>::new());

    view! {
        <h1>"Menu"</h1>
        <p>"placeholder nav page"</p>
        <div id="menu_container">
        //todo
        /*
        {for_leptos!(list, item =>
            <div>{item.text.clone()}</div>
        )}
        */
        </div>
    }
}
