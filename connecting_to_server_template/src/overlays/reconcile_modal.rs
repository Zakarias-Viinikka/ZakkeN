use leptos::prelude::*;

use crate::overlays::full_screen_blocker::FullScreenBlocker;
use crate::reconcile::plan::{ConflictChoice, PageDiffSummary, ReconcileDecisions, ReconcilePlan};

const RM_CSS: &str = include_str!("../../css/reconcile_modal.css");

#[component]
pub fn ReconcileModal(
    plan: Signal<Option<ReconcilePlan>>,
    error: Signal<Option<String>>,
    on_confirm: Callback<ReconcileDecisions>,
    on_cancel: Callback<()>,
) -> impl IntoView {
    view! {
        <style>{RM_CSS}</style>
        <Show when=move || plan.get().is_some() fallback=|| ()>
            {
                let p = plan.get_untracked().unwrap();
                view! {
                    <ReconcileModalBody
                        plan=p
                        error=error
                        on_confirm=on_confirm
                        on_cancel=on_cancel
                    />
                }
            }
        </Show>
    }
}

#[component]
fn ReconcileModalBody(
    plan: ReconcilePlan,
    error: Signal<Option<String>>,
    on_confirm: Callback<ReconcileDecisions>,
    on_cancel: Callback<()>,
) -> impl IntoView {
    let choices = RwSignal::new(ReconcileDecisions::new());

    let plan_server = plan.clone();
    let plan_client = plan.clone();
    let plan_confirm = plan.clone();
    let plan_rows = plan.clone();

    let use_server_all = move |_| {
        let mut map = ReconcileDecisions::new();
        for c in &plan_server.conflicts {
            map.insert(c.page_id.clone(), ConflictChoice::Server);
        }
        choices.set(map);
    };
    let use_client_all = move |_| {
        let mut map = ReconcileDecisions::new();
        for c in &plan_client.conflicts {
            map.insert(c.page_id.clone(), ConflictChoice::Client);
        }
        choices.set(map);
    };
    let confirm_disabled =
        move || every_conflict_needs_a_choice(&plan_confirm.conflicts, &choices.get());

    view! {
        <FullScreenBlocker>
            <div class="rm-card">
                <div class="rm-head">
                    <span class="rm-title">"Reconcile"</span>
                </div>
                <div class="rm-body">
                    <ReconcileSummary plan=plan.clone() />
                    {move || error.get().map(|e| view! { <div class="rm-error">{e}</div> })}
                    <div class="rm-all-buttons">
                        <button class="btn" on:click=use_server_all>"Use server for all"</button>
                        <button class="btn" on:click=use_client_all>"Use mine for all"</button>
                    </div>
                    {plan_rows.conflicts.iter().map(|c| view! {
                        <ConflictRow summary=c.clone() choices=choices />
                    }).collect_view()}
                    <div class="rm-placeholder">"aligned rows preview"</div>
                </div>
                <div class="rm-foot">
                    <button class="btn" on:click=move |_| on_cancel.run(())>"Cancel"</button>
                    <button class="btn" disabled=confirm_disabled on:click=move |_| {
                        on_confirm.run(choices.get_untracked());
                    }>"Confirm"</button>
                </div>
            </div>
        </FullScreenBlocker>
    }
}

fn every_conflict_needs_a_choice(
    conflicts: &[PageDiffSummary],
    chosen: &ReconcileDecisions,
) -> bool {
    conflicts
        .iter()
        .any(|c| !chosen.contains_key(&c.page_id))
}

#[component]
fn ReconcileSummary(plan: ReconcilePlan) -> impl IntoView {
    view! {
        <div class="rm-summary">
            <div class="rm-summary-cell">"From server"<strong>{plan.take_from_server.len()}</strong></div>
            <div class="rm-summary-cell">"To server"<strong>{plan.push_to_server.len()}</strong></div>
            <div class="rm-summary-cell">"Identical"<strong>{plan.identical.len()}</strong></div>
            <div class="rm-summary-cell">"Disable local"<strong>{plan.disable_locally.len()}</strong></div>
            <div class="rm-summary-cell">"Disable server"<strong>{plan.disable_on_server.len()}</strong></div>
        </div>
    }
}

#[component]
fn ConflictRow(
    summary: PageDiffSummary,
    choices: RwSignal<ReconcileDecisions>,
) -> impl IntoView {
    let page_id_s = summary.page_id.clone();
    let page_id_c = summary.page_id.clone();
    let page_id_show = summary.page_id.clone();
    let title = summary.client_title.clone();
    let server_title = summary.server_title.clone();
    let oc = summary.only_client;
    let os = summary.only_server;
    let ch = summary.changed;
    let mv = summary.moved;
    view! {
        <div class="rm-conflict">
            <div class="rm-conflict-head">
                <div class="rm-titles">
                    <div>"mine: " <strong>{title}</strong></div>
                    <div>"server: " <strong>{server_title}</strong></div>
                </div>
                <div class="rm-counts">
                    "only_client=" {oc} " · only_server=" {os}
                    " · changed=" {ch} " · moved=" {mv}
                </div>
            </div>
            <div class="rm-choice-row">
                <button class="btn" on:click=move |_| {
                    choices.update(|m| { m.insert(page_id_s.clone(), ConflictChoice::Server); });
                }>"Use server"</button>
                <button class="btn" on:click=move |_| {
                    choices.update(|m| { m.insert(page_id_c.clone(), ConflictChoice::Client); });
                }>"Use mine"</button>
                <span class="rm-choice-label">
                    {move || match choices.get().get(&page_id_show) {
                        Some(ConflictChoice::Server) => "server",
                        Some(ConflictChoice::Client) => "mine",
                        None => "unpicked",
                    }}
                </span>
            </div>
        </div>
    }
}