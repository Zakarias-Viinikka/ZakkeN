//! Caller pov: the page's render once loading finished. Shows one card
//! per mismatched page, tracks the user's choices, opens the diff
//! overlay on demand, and — when the user clicks Finish — runs the
//! finish-reconcile sequence.

use std::collections::HashMap;

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::leptos_components::dev_panel::sidebar_buttons::reload_the_page;
use crate::leptos_components::small_components::popup::show_popup;

use super::diff_overlay::DiffOverlay;
use super::finish_reconcile::finish_reconcile;
use super::mismatched_card::MismatchedCard;
use super::phase::{ClientOnlyPage, MismatchedPage};
use super::resolve_choice::ResolveChoice;

const CSS: &str = r#"
.rp-meta {
    display: flex;
    gap: 28px;
    font-size: 12px;
    color: rgba(228, 233, 242, 0.55);
    margin-bottom: 20px;
    font-family: ui-monospace, monospace;
}
.rp-meta strong {
    color: #e4e9f2;
    font-weight: 500;
    margin-right: 6px;
}
.rp-summary {
    font-size: 14px;
    color: rgba(228, 233, 242, 0.8);
    margin-bottom: 18px;
}
.rp-summary .rp-count-differ {
    color: #e4e9f2;
    font-weight: 500;
}
.rp-summary .rp-count-client {
    color: #8ab4ff;
    font-weight: 500;
}
.rp-empty {
    padding: 24px 20px;
    background: #1c2433;
    border: 1px solid rgba(228, 233, 242, 0.09);
    border-radius: 10px;
    color: rgba(228, 233, 242, 0.72);
    font-size: 14px;
    margin-bottom: 20px;
}
.rp-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 14px;
    margin-bottom: 28px;
}
.rp-actions {
    display: flex;
    gap: 12px;
    align-items: center;
    padding-top: 20px;
    border-top: 1px solid rgba(228, 233, 242, 0.09);
}
.rp-btn {
    font-family: inherit;
    font-size: 13px;
    padding: 9px 18px;
    border: 1px solid rgba(228, 233, 242, 0.15);
    border-radius: 6px;
    background: transparent;
    color: #e4e9f2;
    cursor: pointer;
    transition: border-color 120ms ease, background 120ms ease;
}
.rp-btn:hover:not(:disabled) {
    border-color: rgba(228, 233, 242, 0.35);
}
.rp-btn:disabled {
    opacity: 0.35;
    cursor: not-allowed;
}
.rp-btn-primary {
    background: #8ab4ff;
    border-color: #8ab4ff;
    color: #0a0e18;
    font-weight: 600;
}
.rp-btn-primary:hover:not(:disabled) {
    background: #a4c5ff;
    border-color: #a4c5ff;
}
.rp-count-note {
    font-size: 12px;
    color: rgba(228, 233, 242, 0.5);
    margin-left: auto;
}
"#;

#[component]
pub fn ReadyView(
    target_drift_version: String,
    target_reconcile_version: String,
    mismatched: Vec<MismatchedPage>,
    client_only: Vec<ClientOnlyPage>,
    mount_fingerprints: HashMap<String, String>,
    user_id: String,
    time: String,
) -> impl IntoView {
    let choices: RwSignal<HashMap<String, ResolveChoice>> = RwSignal::new(HashMap::new());
    let open_diff: RwSignal<Option<String>> = RwSignal::new(None);
    let finishing: RwSignal<bool> = RwSignal::new(false);

    let page_ids: Vec<String> = mismatched.iter().map(|p| p.page_id.clone()).collect();
    let mismatched_for_cards = mismatched.clone();
    let mismatched_for_diff = mismatched.clone();
    let mismatched_for_finish = mismatched.clone();
    let client_only_for_finish = client_only.clone();
    let mount_fingerprints_for_finish = mount_fingerprints.clone();
    let target_dv = target_drift_version.clone();
    let target_rv = target_reconcile_version.clone();
    let user_id_for_finish = user_id.clone();
    let time_for_finish = time.clone();

    let every_resolved = {
        let ids = page_ids.clone();
        Signal::derive(move || {
            let ch = choices.get();
            ids.iter().all(|id| ch.contains_key(id))
        })
    };

    let resolved_count = {
        let ids = page_ids.clone();
        Signal::derive(move || {
            let ch = choices.get();
            ids.iter().filter(|id| ch.contains_key(*id)).count()
        })
    };

    let total = mismatched.len();
    let co_count = client_only.len();
    let has_mismatched = total > 0;

    view! {
        <style>{CSS}</style>

        <div class="rp-meta">
            <span><strong>"drift"</strong>{target_drift_version.clone()}</span>
            <span><strong>"reconcile"</strong>{target_reconcile_version.clone()}</span>
        </div>

        <div class="rp-summary">
            <span class="rp-count-differ">{total}</span>
            " mismatched "
            {if total == 1 { "page" } else { "pages" }}
            {if co_count > 0 {
                Some(view! {
                    ", plus "
                    <span class="rp-count-client">{co_count}</span>
                    " new "
                    {if co_count == 1 { "page" } else { "pages" }}
                    " that will be added"
                })
            } else {
                None
            }}
        </div>

        {if mismatched_for_cards.is_empty() {
            view! {
                <div class="rp-empty">
                    "Nothing to resolve. Click Finish to send the new pages and "
                    "wipe this client so it adopts the server's dataset."
                </div>
            }.into_any()
        } else {
            view! {
                <div class="rp-grid">
                    {mismatched_for_cards
                        .into_iter()
                        .map(|page| {
                            let pid = page.page_id.clone();
                            let pid_for_choice = pid.clone();
                            let pid_for_open = pid.clone();
                            let pid_for_mine = pid.clone();
                            let pid_for_server = pid.clone();
                            let first_client =
                                page.client_blocks.first().cloned().unwrap_or_default();
                            let choice = Signal::derive(move || {
                                choices.get().get(&pid_for_choice).cloned()
                            });
                            view! {
                                <MismatchedCard
                                    page_id=pid
                                    client_block_count=page.client_blocks.len()
                                    server_block_count=page.server_blocks.len()
                                    first_client_block=first_client
                                    choice=choice
                                    on_pick_mine=Callback::new(move |_: ()| {
                                        choices.update(|c| {
                                            c.insert(pid_for_mine.clone(), ResolveChoice::Mine);
                                        });
                                    })
                                    on_pick_server=Callback::new(move |_: ()| {
                                        choices.update(|c| {
                                            c.insert(pid_for_server.clone(), ResolveChoice::Server);
                                        });
                                    })
                                    on_open_diff=Callback::new(move |_: ()| {
                                        open_diff.set(Some(pid_for_open.clone()));
                                    })
                                />
                            }
                        })
                        .collect_view()}
                </div>
            }.into_any()
        }}

        <div class="rp-actions">
            <button
                class="rp-btn rp-btn-primary"
                disabled=move || (has_mismatched && !every_resolved.get()) || finishing.get()
                on:click=move |_| {
                    let choices_now = choices.get();
                    if open_diff.get().is_some() {
                        return;
                    }
                    finishing.set(true);
                    let m = mismatched_for_finish.clone();
                    let c = client_only_for_finish.clone();
                    let mf = mount_fingerprints_for_finish.clone();
                    let dv = target_dv.clone();
                    let rv = target_rv.clone();
                    let uid = user_id_for_finish.clone();
                    let t = time_for_finish.clone();
                    spawn_local(async move {
                        match finish_reconcile(dv, rv, m, c, choices_now, mf, uid, t).await {
                            Ok(()) => reload_the_page(),
                            Err(e) => {
                                show_popup(format!("reconcile failed: {e}"));
                                finishing.set(false);
                            }
                        }
                    });
                }
            >
                {move || if finishing.get() {
                    "Finishing..."
                } else {
                    "Finish reconcile"
                }}
            </button>
            {move || {
                let n = resolved_count.get();
                let t = total;
                if t == 0 {
                    None
                } else {
                    Some(view! {
                        <span class="rp-count-note">
                            {format!("{n} of {t} resolved")}
                        </span>
                    })
                }
            }}
        </div>

        {move || {
            let open = open_diff.get();
            let Some(pid) = open else {
                return ().into_any();
            };
            let Some(page) = mismatched_for_diff.iter().find(|p| p.page_id == pid) else {
                return ().into_any();
            };
            let pid_for_done = pid.clone();
            let on_close = Callback::new(move |_: ()| {
                open_diff.set(None);
            });
            let on_done = Callback::new(move |blocks: Vec<String>| {
                choices.update(|c| {
                    c.insert(pid_for_done.clone(), ResolveChoice::Merged(blocks));
                });
                open_diff.set(None);
            });
            view! {
                <DiffOverlay
                    page_id=pid
                    left_blocks=page.client_blocks.clone()
                    right_blocks=page.server_blocks.clone()
                    on_close=on_close
                    on_done=on_done
                />
            }
            .into_any()
        }}
    }
}