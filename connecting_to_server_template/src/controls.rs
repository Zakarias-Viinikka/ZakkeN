use crate::client_server_coms::{ComsHandleForClient, ComsHandleForServer};
use crate::protocol::LoveLetter;
use crate::CustomButton;
use leptos::prelude::*;
use leptos::task::spawn_local;

fn spawn_verb<F>(fut: F)
where
    F: std::future::Future<Output = Result<(), crate::data_ops::VerbError>> + 'static,
{
    spawn_local(async move {
        if let Err(e) = fut.await {
            match e {
                crate::data_ops::VerbError::Refused(m) => {
                    crate::cool_popup::show_popup_as(crate::cool_popup::PopupKind::Warning, m);
                }
                crate::data_ops::VerbError::Failed(m) => {
                    crate::cool_popup::show_popup_as(crate::cool_popup::PopupKind::Error, m);
                }
            }
        }
    });
}

#[component]
pub fn Controls(coms: ComsHandleForClient, server_coms: ComsHandleForServer) -> impl IntoView {
    let _ = server_coms;
    let coms_connect = coms.clone();
    let coms_wipe = coms.clone();

    let coms_adopt = coms.clone();
    let coms_adopt_wipe = coms.clone();
    let coms_wipe_local = coms.clone();
    let coms_reconcile = coms.clone();
    let coms_turn_on = coms.clone();
    let coms_turn_off = coms.clone();
    let coms_arm = coms.clone();
    let coms_other_wipe = coms.clone();
    let coms_push = coms;
    let next_letter_id = RwSignal::new(1i64);

    let client_disabled = Signal::derive(move || !crate::server_link::client_started_signal().get());
    let powered_off = crate::server::powered_signal();
    let powered_on = crate::server::powered_signal();
    let server_off = Signal::derive(move || !powered_off.get());
    let server_on = Signal::derive(move || powered_on.get());

    view! {
        <div class="controls">
            <div class="controls-section">
                <h3 class="controls-title">"Client"</h3>
                <hr class="controls-hr"/>
                <CustomButton
                    label="Connect to server"
                    disabled=client_disabled
                    on_click=move |_| {
                        if crate::server_link::current_conn_state()
                            == crate::server_link::ConnState::Connected
                        {
                            crate::cool_popup::show_popup("Already connected.");
                            return;
                        }
                        if crate::server_link::current_restart_in_flight() {
                            crate::cool_popup::show_popup("Already connecting.");
                            return;
                        }
                        let coms = coms_connect.clone();
                        spawn_local(async move {
                            let outcome = crate::server_link::restart_sync_stack(coms).await;
                            crate::data_ops::announce_connect_outcome(outcome);
                        });
                    }
                />
                <CustomButton
                    label="Pause sync"
                    disabled=client_disabled
                    on_click=move |_| {
                        if let Err(e) = crate::data_ops::pause() {
                            crate::cool_popup::show_popup_as(crate::cool_popup::PopupKind::Warning, e);
                        }
                    }
                />
            </div>
            <div class="controls-section reconcile-actions">
                <h3 class="controls-title">"Stopped sync: pick one"</h3>
                <hr class="controls-hr"/>
                <CustomButton
                    label="Reconcile"
                    disabled=client_disabled
                    on_click=move |_| {
                        let coms = coms_reconcile.clone();
                        spawn_local(async move {
                            match crate::reconcile::reconcile_plan(coms).await {
                                Ok(Some(plan)) => {
                                    crate::reconcile::plan_signal().set(Some(plan));
                                    crate::reconcile::error_signal().set(None);
                                }
                                Ok(None) => {}
                                Err(crate::data_ops::VerbError::Refused(m)) => {
                                    crate::cool_popup::show_popup_as(
                                        crate::cool_popup::PopupKind::Warning,
                                        m,
                                    );
                                }
                                Err(crate::data_ops::VerbError::Failed(m)) => {
                                    crate::cool_popup::show_popup_as(
                                        crate::cool_popup::PopupKind::Error,
                                        m,
                                    );
                                }
                            }
                        });
                    }
                />
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
            </div>
            <div class="controls-section">
                <h3 class="controls-title">"Popups"</h3>
                <hr class="controls-hr"/>
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
            </div>
            <div class="controls-section">
                <h3 class="controls-title">"Server"</h3>
                <hr class="controls-hr"/>
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
            </div>
        </div>
    }
}