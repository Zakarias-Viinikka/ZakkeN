use crate::client::Client;
use crate::client_server_coms::{self, ComsHandleForClient};
use crate::controls::Controls;
use crate::cool_popup::CoolPopup;
use crate::overlays::{BusyOverlay, ReconcileModal};
use crate::server::panel::ServerPanel;
use crate::server::Server;
use leptos::prelude::*;

fn turn_server_on_when_started(
    server_started: ReadSignal<bool>,
    coms: ComsHandleForClient,
) {
    Effect::new(move |_| {
        if server_started.get() {
            coms.clone().turn_on_server();
        }
    });
}

fn connect_once_when_client_starts(coms: ComsHandleForClient) {
    let auto_connect_seen = RwSignal::new(false);
    Effect::new(move |_| {
        if !crate::client::client_started_signal().get() {
            return;
        }
        if auto_connect_seen.get_untracked() {
            return;
        }
        auto_connect_seen.set(true);
        crate::data_ops::connect_to_server_and_announce(coms.clone());
    });
}

fn retry_reconnect_when_offline(coms: ComsHandleForClient) {
    use crate::owner_of_interacting_with_server::conn_state::ConnState;
    let retry_running = RwSignal::new(false);
    Effect::new(move |_| {
        let running = retry_running.get();
        if !crate::client::client_started_signal().get() {
            return;
        }
        let conn = crate::owner_of_interacting_with_server::conn_state::conn_state_signal().get();
        let locked =
            crate::owner_of_interacting_with_server::locked::hard_locked_signal().get();
        let should_retry =
            !locked && (conn == ConnState::Offline || conn == ConnState::FailedToConnect);
        if !should_retry || running {
            return;
        }
        retry_running.set(true);
        let coms = coms.clone();
        let retry_running_for_loop = retry_running.clone();
        leptos::task::spawn_local(async move {
            loop {
                {
                    let (tx, rx) = futures::channel::oneshot::channel::<()>();
                    leptos::prelude::set_timeout(
                        move || {
                            let _ = tx.send(());
                        },
                        std::time::Duration::from_millis(5000),
                    );
                    let _ = rx.await;
                }
                if !crate::client::client_started_signal().get_untracked() {
                    break;
                }
                if crate::owner_of_interacting_with_server::locked::is_locked() {
                    break;
                }
                let conn = crate::owner_of_interacting_with_server::conn_state::conn_state_signal().get_untracked();
                if conn != ConnState::Offline && conn != ConnState::FailedToConnect {
                    break;
                }
                crate::cool_popup::show_popup_as(
                    crate::cool_popup::PopupKind::Warning,
                    "Trying to reconnect…",
                );
                let outcome = crate::owner_of_interacting_with_server::restart::restart_sync_stack(coms.clone()).await;
                match outcome {
                    crate::owner_of_interacting_with_server::restart::RestartOutcome::Failed(_) => {
                        // stay in the loop, retry again in 5s
                    }
                    other => {
                        // JustChanged pops "Server dataset changed. Sync is
                        // stopped until you Reconcile or Adopt." — which is
                        // exactly the case where the client was offline when
                        // another client wiped the server, and the retry
                        // reconnect is the first time it hears about it.
                        crate::data_ops::announce_connect_outcome(other);
                        break;
                    }
                }
            }
            retry_running_for_loop.set(false);
        });
    });
}

fn client_title() -> Signal<String> {
    use crate::owner_of_interacting_with_server::conn_state::ConnState;
    use crate::sync_mode::SyncMode;
    Signal::derive(move || {
        if crate::owner_of_interacting_with_server::locked::hard_locked_signal().get() {
            return "Client — Locked (restart)".to_string();
        }
        let conn = crate::owner_of_interacting_with_server::conn_state::conn_state_signal().get();
        let mode = crate::sync_mode::sync_mode_signal().get();
        let state = match (conn, mode) {
            (ConnState::Connected, SyncMode::NeedsReconcile) => "Lonewolf",
            (ConnState::Connected, SyncMode::Connected) => "Connected",
            (ConnState::Connecting, _) => "Connecting",
            (ConnState::FailedToConnect, _) => "Failed to connect",
            (ConnState::Offline, _) => "Offline",
        };
        format!("Client — {state}")
    })
}

#[component]
pub fn AppMain() -> impl IntoView {
    let (server_started, set_server_started) = signal(false);
    let (client_handle, server_handle, client_rx, server_rx) = client_server_coms::new();
    let client_handle_for_controls = client_handle.clone();
    let client_handle_for_modal = client_handle.clone();
    let client_handle_for_server_effect = client_handle.clone();
    let client_handle_for_auto = client_handle.clone();
    let client_handle_for_retry = client_handle.clone();
    let server_handle_for_controls = server_handle.clone();

    turn_server_on_when_started(server_started, client_handle_for_server_effect);
    connect_once_when_client_starts(client_handle_for_auto);
    retry_reconnect_when_offline(client_handle_for_retry);

    let plan_sig = Signal::derive(move || crate::reconcile::plan_signal().get());
    let error_sig = Signal::derive(move || crate::reconcile::error_signal().get());

    let ch_confirm = client_handle_for_modal.clone();
    let on_confirm = Callback::new(move |decisions: crate::reconcile::plan::ReconcileDecisions| {
        crate::reconcile::modal::apply_decisions_from_modal(ch_confirm.clone(), decisions);
    });
    let ch_cancel = client_handle_for_modal;
    let on_cancel = Callback::new(move |_: ()| {
        crate::reconcile::modal::cancel_from_modal(ch_cancel.clone());
    });

    view! {
        <div class="sides">
            <Client handle=client_handle rx=client_rx>
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
                            <crate::client::panel::ClientPanel/>
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