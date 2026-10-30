use crate::client_server_coms::{ComsHandleForClient, ComsHandleForServer};
use crate::data_ops::verb_error::{announce_verb_error, VerbError};
use crate::protocol::LoveLetter;
use crate::CustomButton;
use leptos::prelude::*;
use leptos::task::spawn_local;

fn spawn_verb<F>(fut: F)
where
    F: std::future::Future<Output = Result<(), VerbError>> + 'static,
{
    spawn_local(async move {
        if let Err(e) = fut.await {
            announce_verb_error(e);
        }
    });
}

#[component]
fn ControlsSection(
    title: &'static str,
    #[prop(default = "")] extra_class: &'static str,
    children: Children,
) -> impl IntoView {
    let class = if extra_class.is_empty() {
        "controls-section".to_string()
    } else {
        format!("controls-section {}", extra_class)
    };
    view! {
        <div class=class>
            <h3 class="controls-title">{title}</h3>
            <hr class="controls-hr"/>
            {children()}
        </div>
    }
}

#[component]
fn ClientControls(coms: ComsHandleForClient) -> impl IntoView {
    let coms_connect = coms;
    let client_disabled = Signal::derive(move || !crate::client::client_started_signal().get());

    let connect = move |_| {
        if crate::server_link::conn_state::current_conn_state()
            == crate::server_link::conn_state::ConnState::Connected
        {
            crate::cool_popup::show_popup("Already connected.");
            return;
        }
        if crate::server_link::restart::current_restart_in_flight() {
            crate::cool_popup::show_popup("Already connecting.");
            return;
        }
        crate::data_ops::connect_to_server_and_announce(coms_connect.clone());
    };

    let pause = move |_| {
        if let Err(e) = crate::data_ops::pause() {
            crate::cool_popup::show_popup_as(crate::cool_popup::PopupKind::Warning, e);
        }
    };

    view! {
        <ControlsSection title="Client">
            <CustomButton label="Connect to server" disabled=client_disabled on_click=connect />
            <CustomButton label="Pause sync" disabled=client_disabled on_click=pause />
        </ControlsSection>
    }
}

#[component]
fn StoppedSyncControls(coms: ComsHandleForClient) -> impl IntoView {
    let coms_reconcile = coms.clone();
    let coms_adopt = coms.clone();
    let coms_adopt_wipe = coms.clone();
    let coms_wipe_local = coms;
    let client_disabled = Signal::derive(move || !crate::client::client_started_signal().get());

    let reconcile_click = move |_| {
        let coms = coms_reconcile.clone();
        spawn_local(async move {
            match crate::reconcile::reconcile_plan(coms).await {
                Ok(Some(plan)) => crate::reconcile::modal::open_modal(plan),
                Ok(None) => {}
                Err(e) => announce_verb_error(e),
            }
        });
    };

    view! {
        <ControlsSection title="Stopped sync: pick one" extra_class="reconcile-actions">
            <CustomButton label="Reconcile" disabled=client_disabled on_click=reconcile_click />
            <CustomButton
                label="Adopt my data"
                disabled=client_disabled
                on_click=move |_| {
                    let coms = coms_adopt.clone();
                    spawn_verb(crate::data_ops::adopt_local(coms));
                }
            />
            <CustomButton
                label="Wipe server & start from mine"
                disabled=client_disabled
                on_click=move |_| {
                    let coms = coms_adopt_wipe.clone();
                    spawn_verb(crate::data_ops::adopt_local(coms));
                }
            />
            <CustomButton
                label="Wipe mine & start from server"
                disabled=client_disabled
                on_click=move |_| {
                    let coms = coms_wipe_local.clone();
                    spawn_verb(crate::data_ops::wipe_local_and_pull_from_server(coms));
                }
            />
        </ControlsSection>
    }
}

#[component]
fn PopupControls() -> impl IntoView {
    view! {
        <ControlsSection title="Popups">
            <CustomButton
                label="Show popup"
                on_click=move |_| crate::cool_popup::show_popup("Hello from cool popup")
            />
            <CustomButton
                label="Show error"
                on_click=move |_| crate::cool_popup::show_popup_as(
                    crate::cool_popup::PopupKind::Error,
                    "Something went wrong",
                )
            />
        </ControlsSection>
    }
}

#[component]
fn ServerControls(coms: ComsHandleForClient) -> impl IntoView {
    let coms_wipe = coms.clone();
    let coms_turn_on = coms.clone();
    let coms_turn_off = coms.clone();
    let coms_arm = coms.clone();
    let coms_other_wipe = coms.clone();
    let coms_push = coms;
    let next_letter_id = RwSignal::new(1i64);
    let client_disabled = Signal::derive(move || !crate::client::client_started_signal().get());
    let powered_off = crate::server::power::powered_signal();
    let powered_on = crate::server::power::powered_signal();
    let server_off = Signal::derive(move || !powered_off.get());
    let server_on = Signal::derive(move || powered_on.get());

    view! {
        <ControlsSection title="Server">
            <CustomButton
                label="Turn off server"
                disabled=server_off
                on_click=move |_| coms_turn_off.turn_off_server()
            />
            <CustomButton
                label="Start server"
                disabled=server_on
                on_click=move |_| coms_turn_on.turn_on_server()
            />
            <CustomButton
                label="Wipe server"
                disabled=client_disabled
                on_click=move |_| {
                    let coms = coms_wipe.clone();
                    spawn_verb(crate::data_ops::wipe_server(coms));
                }
            />
            <CustomButton
                label="Push pretend letter"
                disabled=server_off
                on_click=move |_| {
                    let id = next_letter_id.get_untracked();
                    next_letter_id.set(id + 1);
                    coms_push.push_pretend_letter(LoveLetter::pretend_edit(
                        "page_a",
                        format!("push #{id}"),
                    ));
                }
            />
            <CustomButton
                label="Fail next request"
                disabled=server_off
                on_click=move |_| coms_arm.arm_fault()
            />
            <CustomButton
                label="Other client wipes"
                disabled=server_off
                on_click=move |_| coms_other_wipe.other_client_wipes()
            />
        </ControlsSection>
    }
}

#[component]
pub fn Controls(coms: ComsHandleForClient, server_coms: ComsHandleForServer) -> impl IntoView {
    let _ = server_coms;
    let coms_client = coms.clone();
    let coms_stopped = coms.clone();
    let coms_server = coms;
    view! {
        <div class="controls">
            <ClientControls coms=coms_client />
            <StoppedSyncControls coms=coms_stopped />
            <PopupControls />
            <ServerControls coms=coms_server />
        </div>
    }
}