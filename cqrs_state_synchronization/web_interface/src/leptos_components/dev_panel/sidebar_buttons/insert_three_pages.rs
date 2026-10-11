use leptos::prelude::*;
use leptos::reactive::spawn_local;

use crate::{
    db::dev_actions::insert_three_pages,
    leptos_components::{
        dev_panel::sidebar_buttons::reload_the_page,
        small_components::popup::create_popup,
    },
};

#[component]
pub fn InsertThreePagesButton() -> impl IntoView {
    view! {
        <button on:click=move |_| {
            spawn_local(async move {
                match insert_three_pages().await {
                    Ok(()) => reload_the_page(),
                    Err(e) => create_popup(format!("Error: {:?}", e)),
                }
            });
        }>"insert three pages"</button>
    }
}
