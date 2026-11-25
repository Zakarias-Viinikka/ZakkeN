use leptos::prelude::*;
use shared_internals::ClientCommand;

use crate::complex_state::router_handle;
use crate::leptos_components::small_components::popup::show_popup;

#[component]
pub fn ImpOrExport() -> impl IntoView {
    let on_export = move |_| {
        show_popup("Started export...".to_string());
        leptos::task::spawn_local(async move {
            match router_handle::send(ClientCommand::Export).await {
                Ok(_) => {
                    show_popup("Export saved to Downloads folder".to_string());
                }
                Err(e) => {
                    show_popup(format!("Export failed: {e:?}"));
                }
            }
        });
    };

    view! {
        <div style="padding: 24px;">
            <h2>"Import / Export"</h2>
            <div style="margin-top: 16px;">
                <button
                    style="padding: 8px 16px; cursor: pointer;"
                    on:click=on_export
                >
                    "Export All Pages"
                </button>
            </div>
        </div>
    }
}