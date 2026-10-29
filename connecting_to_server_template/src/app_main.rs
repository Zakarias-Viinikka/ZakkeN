use crate::client::Client;
use crate::client_server_coms;
use crate::cool_popup::CoolPopup;
use crate::overlays::{BusyOverlay, ReconcileModal};
use crate::controls::Controls;
use crate::server::{Server, ServerPanel};
use leptos::prelude::*;

#[component]
pub fn AppMain() -> impl IntoView {
    let (server_started, set_server_started) = signal(false);
    let (client_handle, server_handle, client_rx, server_rx) = client_server_coms::new();
    let client_handle_for_controls = client_handle.clone();
    let client_handle_for_modal = client_handle.clone();
    let client_handle_for_effect = client_handle.clone();
    let server_handle_for_controls = server_handle.clone();

    Effect::new(move |_| {
        if server_started.get() {
            client_handle_for_effect.clone().turn_on_server();
        }
    });

    let client_handle_for_auto = client_handle.clone();
    let auto_connect_seen = RwSignal::new(false);
    Effect::new(move |_| {
        if !crate::server_link::client_started_signal().get() {
            return;
        }
        if auto_connect_seen.get_untracked() {
            return;
        }
        auto_connect_seen.set(true);
        let coms = client_handle_for_auto.clone();
        leptos::task::spawn_local(async move {
            let outcome = crate::server_link::restart_sync_stack(coms).await;
            crate::data_ops::announce_connect_outcome(outcome);
        });
    });

    let plan_sig = Signal::derive(move || crate::reconcile::plan_signal().get());
    let error_sig = Signal::derive(move || crate::reconcile::error_signal().get());


    let ch_confirm = client_handle_for_modal.clone();
    let on_confirm = Callback::new(move |decisions: crate::reconcile::ReconcileDecisions| {
        let coms = ch_confirm.clone();
        leptos::task::spawn_local(async move {
            match crate::reconcile::reconcile_apply(coms, decisions).await {
                Ok(()) => crate::reconcile::clear_modal(),
                Err(e) => crate::reconcile::error_signal().set(Some(e.to_string())),
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
                    when=move || crate::server_link::client_started_signal().get()
                    fallback=move || view! {
                        <ClientFallback on_start=move |_| {
                            crate::server_link::client_started_signal().set(true)
                        }/>
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