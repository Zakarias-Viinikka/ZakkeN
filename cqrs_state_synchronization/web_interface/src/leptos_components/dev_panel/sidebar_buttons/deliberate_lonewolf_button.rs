use leptos::prelude::*;
use leptos::reactive::spawn_local;

use crate::db::dev_actions;
use crate::leptos_components::small_components::popup::show_popup;

#[component]
pub fn DeliberateLonewolfButton() -> impl IntoView {
    view! {
        <button on:click=move |_| {
            spawn_local(async move {
                if let Err(e) = dev_actions::enter_lone_wolf().await {
                    show_popup(format!("enter lonewolf: {e:?}"));
                }
            });
        }>"deliberate lonewolf"</button>
    }
}