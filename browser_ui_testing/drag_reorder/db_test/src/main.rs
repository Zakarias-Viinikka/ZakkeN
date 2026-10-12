mod schema;
mod shared_structs;

use leptos::prelude::*;
use schema::{get_all_words, init_db};
use shared_structs::WordHolder;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let (words, set_words) = signal(Vec::<WordHolder>::new());

    leptos::task::spawn_local(async move {
        if let Err(e) = init_db().await {
            leptos::logging::error!("init_db failed: {:?}", e);
            return;
        }
        match get_all_words().await {
            Ok(w) => set_words.set(w),
            Err(e) => leptos::logging::error!("get_all_words failed: {:?}", e),
        }
    });

    view! {
        <div class="word-list">
            {move || words.get().into_iter().map(|w| view! {
                <div class="word-box">
                    <div class="handle">"::"</div>
                    <div class="word-text">{w.word.clone()}</div>
                </div>
            }).collect_view()}
        </div>
    }
}