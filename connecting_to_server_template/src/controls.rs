use crate::client_server_coms::{ComsHandleForClient, ComsHandleForServer};
use crate::protocol::{LoveLetter, UnsentLetter};
use crate::CustomButton;
use leptos::prelude::*;
use leptos::task::spawn_local;

fn spawn_verb<F>(fut: F)
where
    F: std::future::Future<Output = Result<(), String>> + 'static,
{
    spawn_local(async move {
        if let Err(e) = fut.await {
            crate::cool_popup::show_popup(e);
        }
    });
}

#[component]
pub fn Controls(coms: ComsHandleForClient, server_coms: ComsHandleForServer) -> impl IntoView {
    let coms_connect = coms.clone();
    let coms_wipe = coms.clone();
    let coms_resume = coms.clone();
    let coms_adopt = coms.clone();
    let coms_reconcile = coms;
    let server_coms_push = server_coms;
    let next_letter_id = RwSignal::new(1i64);
    view! {
        <div class="controls">
            <div class="controls-section">
                <h3 class="controls-title">"Client"</h3>
                <hr class="controls-hr"/>
                <CustomButton
                    label="Connect to server"
                    on_click=move |_| {
                        let coms = coms_connect.clone();
                        spawn_verb(crate::server_link::restart_sync_stack(coms));
                    }
                />
                <CustomButton
                    label="Pause sync"
                    on_click=move |_| {
                        if let Err(e) = crate::data_ops::pause() {
                            crate::cool_popup::show_popup(e);
                        }
                    }
                />
                <CustomButton
                    label="Resume sync"
                    on_click=move |_| {
                        let coms = coms_resume.clone();
                        spawn_verb(crate::data_ops::resume(coms));
                    }
                />
                <CustomButton
                    label="Adopt my data"
                    on_click=move |_| {
                        let coms = coms_adopt.clone();
                        spawn_verb(crate::data_ops::adopt_local(coms));
                    }
                />
                <CustomButton
                    label="Reconcile"
                    on_click=move |_| {
                        let coms = coms_reconcile.clone();
                        spawn_verb(async move {
                            let plan = crate::reconcile::reconcile_plan(coms).await?;
                            crate::reconcile::plan_signal().set(Some(plan));
                            crate::reconcile::error_signal().set(None);
                            Ok(())
                        });
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
                <CustomButton label="Turn off server"/>
                <CustomButton label="Start server"/>
                <CustomButton
                    label="Wipe server"
                    on_click=move |_| {
                        let coms = coms_wipe.clone();
                        spawn_verb(crate::data_ops::wipe_server(coms));
                    }
                />
                <CustomButton
                    label="Push pretend letter"
                    on_click=move |_| {
                        let id = next_letter_id.get_untracked();
                        next_letter_id.set(id + 1);
                        server_coms_push.push_letter(UnsentLetter {
                            letter_id: id,
                            love_letter: LoveLetter::pretend_edit("page_a", format!("push #{id}")),
                        });
                    }
                />
            </div>
        </div>
    }
}