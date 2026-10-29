use crate::client::Client;
use crate::client_server_coms;
use crate::cool_popup::CoolPopup;
use crate::overlays::{BusyOverlay, ReconcileModal};
use crate::controls::Controls;
use crate::server::{Server, ServerPanel};
use leptos::prelude::*;

#[component]
pub fn AppMain() -> impl IntoView {
    let (client_started, set_client_started) = signal(false);
    let (server_started, set_server_started) = signal(false);
    let (client_handle, server_handle, client_rx, server_rx) = client_server_coms::new();
    let client_handle_for_controls = client_handle.clone();
    let client_handle_for_modal = client_handle.clone();
    let server_handle_for_controls = server_handle.clone();

    let plan_sig = Signal::derive(move || crate::reconcile::plan_signal().get());
    let error_sig = Signal::derive(move || crate::reconcile::error_signal().get());


    let ch_confirm = client_handle_for_modal.clone();
    let on_confirm = Callback::new(move |decisions: crate::reconcile::ReconcileDecisions| {
        let coms = ch_confirm.clone();
        leptos::task::spawn_local(async move {
            match crate::reconcile::reconcile_apply(coms, decisions).await {
                Ok(()) => crate::reconcile::clear_modal(),
                Err(e) => crate::reconcile::error_signal().set(Some(e)),
            }
        });
    });
    let on_cancel = Callback::new(move |_: ()| {
        let coms = client_handle_for_modal.clone();
        leptos::task::spawn_local(async move {
            let _ = crate::reconcile::reconcile_cancel(coms).await;
            crate::reconcile::clear_modal();
        });
    });
    view! {
        <div class="sides">
            <Client handle=client_handle rx=client_rx>
                <Show
                    when=move || client_started.get()
                    fallback=move || view! {
                        <ClientFallback on_start=move |_| set_client_started.set(true)/>
                    }
                >
                    <div class="side">
                        <h2 class="side-title">"Client"</h2>
                        <div class="side-content">
                            <crate::client::ClientPanel/>
                        </div>
                    </div>
                </Show>
            </Client>
            <Server handle=server_handle rx=server_rx started=Signal::derive(move || server_started.get())>
                <Show
                    when=move || server_started.get()
                    fallback=move || view! {
                        <ServerFallback on_start=move |_| set_server_started.set(true)/>
                    }
                >
                    <ServerPanel/>
                </Show>
            </Server>
            <Controls coms=client_handle_for_controls server_coms=server_handle_for_controls/>
        </div>
        <ReconcileModal
            plan=plan_sig
            error=error_sig
            on_confirm=on_confirm
            on_cancel=on_cancel
        />
        <BusyOverlay/>
        <CoolPopup/>
    }
}

#[component]
fn ClientFallback(#[prop(into)] on_start: Callback<()>) -> impl IntoView {
    view! {
        <div class="side-empty">
            <crate::CustomButton label="Start client" on_click=on_start/>
        </div>
    }
}

#[component]
fn ServerFallback(#[prop(into)] on_start: Callback<()>) -> impl IntoView {
    view! {
        <div class="side-empty">
            <crate::CustomButton label="Start server" on_click=on_start/>
        </div>
    }
}