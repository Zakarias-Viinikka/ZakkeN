use leptos::prelude::*;
use leptos::reactive::spawn_local;
use shared_internals::ClientCommand;

use crate::{
    leptos_components::small_components::popup::show_popup,
    complex_state::router_handle,
};

#[component]
pub fn WipeServerButton() -> impl IntoView {
    view! {
        <button on:click=move |_| {
            spawn_local(async move {
                if let Err(e) = router_handle::send(ClientCommand::WipeDataset).await {
                    show_popup(format!("wipe server failed: {e:?}"));
                }
            });
        }>"wipe server"</button>
    }
}