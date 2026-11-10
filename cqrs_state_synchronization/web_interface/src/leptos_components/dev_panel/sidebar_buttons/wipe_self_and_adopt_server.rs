use leptos::prelude::*;
use leptos::reactive::spawn_local;

use crate::{
    db::dev_actions::wipe_local_and_pull_from_server,
    leptos_components::{
        dev_panel::sidebar_buttons::reload_the_page,
        small_components::popup::show_popup,
    },
};

#[component]
pub fn WipeSelfAndAdoptServerButton() -> impl IntoView {
    view! {
        <button on:click=move |_| {
            spawn_local(async move {
                match wipe_local_and_pull_from_server().await {
                    Ok(()) => reload_the_page(),
                    Err(e) => show_popup(format!("Error: {:?}", e)),
                }
            });
        }>"wipe self and adopt server"</button>
    }
}
