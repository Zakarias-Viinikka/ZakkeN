use crate::client::Client;
use crate::client::owner_for_auto_connect::main_for_connecting_the_client_once_when_it_starts;
use crate::client::owner_for_auto_reconnect::main_for_retrying_the_connection_while_offline;
use crate::client_server_coms::{self};
use crate::controls::Controls;
use crate::cool_popup::CoolPopup;
use crate::overlays::{BusyOverlay, ReconcileModal};
use crate::server::Server;
use crate::server::components::panel::ServerPanel;
use crate::server::owner_for_auto_power_on::main_for_powering_the_server_when_the_user_starts_it;
use leptos::prelude::*;

#[component]
pub fn AppMain() -> impl IntoView {
    let (server_started, set_server_started) = signal(false);
    let (client_handle, server_handle, client_rx, server_rx) = client_server_coms::new();

    main_for_powering_the_server_when_the_user_starts_it(server_started, client_handle.clone());
    main_for_connecting_the_client_once_when_it_starts(client_handle.clone());
    main_for_retrying_the_connection_while_offline(client_handle.clone());

    let open_sig = Signal::derive(move || crate::reconcile::open_signal().get());
    let error_sig = Signal::derive(move || crate::reconcile::error_signal().get());

    let ch_confirm = client_handle.clone();
    let on_confirm = Callback::new(move |_: ()| {
        crate::reconcile::apply_from_modal(ch_confirm.clone());
    });
    let ch_cancel = client_handle.clone();
    let on_cancel = Callback::new(move |_: ()| {
        crate::reconcile::cancel_from_modal(ch_cancel.clone());
    });

    view! {
        <div class="sides">
            <Client handle=client_handle.clone() rx=client_rx>
                <Show
                    when=move || crate::client::client_started_signal().get()
                    fallback=move || view! {
                        <ClientFallback on_start=move |_| {
                            crate::client::client_started_signal().set(true)
                        }/>
                    }
                >
                    <div class="side">
                        <h2 class="side-title">{move || client_title().get()}</h2>
                        <div class="side-content">
                            <crate::client::components::panel::ClientPanel/>
                        </div>
                    </div>
                </Show>
            </Client>
            <Server handle=server_handle.clone() rx=server_rx started=Signal::derive(move || server_started.get())>
                <Show
                    when=move || server_started.get()
                    fallback=move || view! {
                        <ServerFallback on_start=move |_| set_server_started.set(true)/>
                    }
                >
                    <ServerPanel/>
                </Show>
            </Server>
            <Controls coms=client_handle.clone() server_coms=server_handle.clone()/>
        </div>
        <ReconcileModal
            open=open_sig
            error=error_sig
            on_confirm=on_confirm
            on_cancel=on_cancel
        />
        <BusyOverlay/>
        <CoolPopup/>
    }
}

fn client_title() -> Signal<String> {
    use crate::shared_owners::client_status::{ClientStatus, client_status_signal};
    let status = client_status_signal();
    Signal::derive(move || {
        let label = match status.get() {
            ClientStatus::NotStarted => "Not started",
            ClientStatus::Locked => "Locked (restart)",
            ClientStatus::Offline => "Offline",
            ClientStatus::Connecting => "Connecting",
            ClientStatus::FailedToConnect => "Failed to connect",
            ClientStatus::Connected => "Connected",
            ClientStatus::NeedsReconcile => "Stopped",
            ClientStatus::Lonewolf => "Lonewolf",
        };
        format!("Client — {label}")
    })
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