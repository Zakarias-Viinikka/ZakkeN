use leptos::prelude::*;
use leptos::reactive::spawn_local;

use crate::{
    db::dev_actions::delete_everything,
    leptos_components::{
        dev_panel::sidebar_buttons::reload_the_page,
        small_components::popup::show_popup,
    },
};

#[component]
pub fn DeleteEverythingButton() -> impl IntoView {
    view! {
        <button on:click=move |_| {
            spawn_local(async move {
                match delete_everything().await {
                    Ok(()) => reload_the_page(),
                    Err(e) => show_popup(format!("Error: {:?}", e)),
                }
            });
        }>"delete everything"</button>
    }
}