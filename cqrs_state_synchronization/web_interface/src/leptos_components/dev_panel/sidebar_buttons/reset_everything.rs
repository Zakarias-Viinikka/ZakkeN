use leptos::prelude::*;
use leptos::reactive::spawn_local;

use crate::{
    db::ui_actions::reset_everything,
    leptos_components::{
        dev_panel::sidebar_buttons::reload_the_page,
        small_components::popup::create_popup,
    },
};

#[component]
pub fn ResetEverythingButton() -> impl IntoView {
    view! {
        <button on:click=move |_| {
            spawn_local(async move {
                match reset_everything().await {
                    Ok(()) => reload_the_page(),
                    Err(e) => create_popup(format!("Error: {:?}", e)),
                }
            });
        }>"reset everything"</button>
    }
}
