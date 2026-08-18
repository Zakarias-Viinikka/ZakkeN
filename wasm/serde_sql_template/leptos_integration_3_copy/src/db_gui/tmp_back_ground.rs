use leptos::prelude::*;

#[component]
pub fn TmpBackGround() -> impl IntoView {
    view! {
        <div style="position:fixed; inset:0; background:#1e1e1e; z-index:9999;"></div>
    }
}
