//! Caller pov: the page's render once loading finished. Owns per-page
//! state (picked side, last viewer output) and orchestrates three sub-
//! views: the numbered chooser + default picker + submit panel when
//! nothing is open, the pick screen when a page is open but has no pick,
//! and the inline viewer host when a page is open and picked. On Submit
//! the viewer's already-produced love letters from every picked page are
//! aggregated, client-only pages contribute CreateNewPage letters, and
//! `finish_reconcile` sends them.

use std::collections::HashMap;
use std::sync::Arc;

use block_diff_viewer_leptos::{
    LoveLetter, PageBlocksForTheViewer, PickedSide, ViewerOutput,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use love_letter::{build_a_love_letter, LoveLetterSketch};
use my_yrs_lib::BossOfYrs;

use crate::leptos_components::dev_panel::sidebar_buttons::reload_the_page;
use crate::leptos_components::small_components::popup::show_popup;

use super::finish_reconcile::finish_reconcile;
use super::inline_diff_viewer_host::InlineDiffViewerHost;
use super::nothing_selected_panel::NothingSelectedPanel;
use super::phase::{ClientOnlyPage, MismatchedPage};
use super::pick_which_side_wins_screen::PickWhichSideWinsScreen;


pub fn summary_of_one_love_letter(l: &LoveLetter) -> String {
    match l {
        LoveLetter::EditBlock { block_id, .. } => format!("EditBlock({block_id})"),
        LoveLetter::RemoveBlock { block_id, .. } => format!("RemoveBlock({block_id})"),
        LoveLetter::CreateNewBlock { .. } => "CreateNewBlock".to_string(),
        LoveLetter::CreateNewBlockWithId { block_id, .. } => {
            format!("CreateNewBlockWithId({block_id})")
        }
        LoveLetter::MoveBlock { block_id, .. } => format!("MoveBlock({block_id})"),
        LoveLetter::DisablePage { page_id, .. } => format!("DisablePage({page_id})"),
        LoveLetter::CreateNewPage { page_id, .. } => format!("CreateNewPage({page_id})"),
    }
}

#[derive(Clone)]
struct PairState {
    page: Arc<MismatchedPage>,
    server_boss: Option<Arc<BossOfYrs>>,
    picked: RwSignal<Option<PickedSide>>,
    last_output: RwSignal<Option<ViewerOutput>>,
}

fn build_pairs(
    mismatched: &[MismatchedPage],
    user_id: &str,
    time: &str,
) -> (Vec<PairState>, Option<String>) {
    let mut pairs = Vec::with_capacity(mismatched.len());
    for page in mismatched {
        let boss_result = my_yrs_lib::doc_from_snapshot(
            page.server_blob.clone(),
            user_id.to_string(),
            page.page_id.clone(),
            time.to_string(),
        );
        let server_boss = match boss_result {
            Ok(b) => Some(Arc::new(b)),
            Err(e) => {
                return (
                    Vec::new(),
                    Some(format!("decode server page {}: {e:?}", page.page_id)),
                );
            }
        };
        pairs.push(PairState {
            page: Arc::new(page.clone()),
            server_boss,
            picked: RwSignal::new(None),
            last_output: RwSignal::new(None),
        });
    }
    (pairs, None)
}

fn server_page_for_viewer(page: &MismatchedPage) -> PageBlocksForTheViewer {
    PageBlocksForTheViewer {
        page_id: page.page_id.clone(),
        is_main_menu_page: false,
        blocks: page.server_blocks.clone(),
        blob: page.server_blob.clone(),
    }
}

fn client_page_for_viewer(page: &MismatchedPage) -> PageBlocksForTheViewer {
    PageBlocksForTheViewer {
        page_id: page.page_id.clone(),
        is_main_menu_page: false,
        blocks: page.client_blocks.clone(),
        blob: page.client_blob.clone(),
    }
}

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
    let total = mismatched.len();
    let co_count = client_only.len();

    let (pairs, fatal) = build_pairs(&mismatched, &user_id, &time);
    if let Some(err) = fatal {
        return view! {
            <div class="rp-meta">
                <span><strong>"drift"</strong>{target_drift_version}</span>
                <span><strong>"reconcile"</strong>{target_reconcile_version}</span>
            </div>
            <div class="rp-fatal">{err}</div>
        }
            .into_any();
    }

    let pairs_signal: RwSignal<Vec<PairState>> = RwSignal::new(pairs);
    let default_picked: RwSignal<Option<PickedSide>> = RwSignal::new(None);
    let selected: RwSignal<Option<String>> = RwSignal::new(None);
    let reviewing: RwSignal<Option<(Vec<(String, Vec<String>)>, Vec<String>)>> =
        RwSignal::new(None);
    let finishing: RwSignal<bool> = RwSignal::new(false);

    let target_dv = target_drift_version.clone();
    let target_rv = target_reconcile_version.clone();
    let mf_for_finish = mount_fingerprints.clone();
    let user_id_for_finish = user_id.clone();
    let time_for_finish = time.clone();
    let client_only_for_finish = client_only.clone();

    let ready_to_submit = {
        let pairs_sig = pairs_signal;
        let default_sig = default_picked;
        Signal::derive(move || {
            let default = default_sig.get();
            pairs_sig.get().iter().all(|p| {
                p.picked.get().is_some() || default.is_some()
            })
        })
    };

    let resolved_count = {
        let pairs_sig = pairs_signal;
        let default_sig = default_picked;
        Signal::derive(move || {
            let default = default_sig.get();
            pairs_sig
                .get()
                .iter()
                .filter(|p| p.picked.get().is_some() || default.is_some())
                .count()
        })
    };

    let on_set_default = Callback::new(move |side: Option<PickedSide>| {
        default_picked.set(side);
    });

    let on_submit_clicked = {
        let pairs_sig = pairs_signal;
        let default_sig = default_picked;
        let co = client_only.clone();
        let review_sig = reviewing;
        Callback::new(move |_: ()| {
            let snapshot = pairs_sig.get_untracked();
            let default = default_sig.get_untracked();
            let mut per_page: Vec<(String, Vec<String>)> = Vec::new();
            for pair in &snapshot {
                let effective = pair.picked.get_untracked().or(default);
                if effective.is_none() {
                    continue;
                }
                let out = pair.last_output.get_untracked();
                match out {
                    Some(o) => {
                        let summaries: Vec<String> =
                            o.love_letters.iter().map(summary_of_one_love_letter).collect();
                        per_page.push((pair.page.page_id.clone(), summaries));
                    }
                    None => {
                        per_page.push((pair.page.page_id.clone(), Vec::new()));
                    }
                }
            }
            let co_ids: Vec<String> =
                co.iter().map(|p| p.page_id.clone()).collect();
            review_sig.set(Some((per_page, co_ids)));
        })
    };

    let on_review_back = Callback::new(move |_: ()| reviewing.set(None));

    let on_send_clicked = {
        let pairs_sig = pairs_signal;
        let default_sig = default_picked;
        let co = client_only_for_finish.clone();
        let dv = target_dv;
        let rv = target_rv;
        let mf = mf_for_finish;
        let uid = user_id_for_finish;
        let t = time_for_finish;
        let finishing_sig = finishing;
        Callback::new(move |_: ()| {
            finishing_sig.set(true);
            let snapshot = pairs_sig.get_untracked();
            let default = default_sig.get_untracked();
            let mut letters: Vec<LoveLetter> = Vec::new();
            for pair in &snapshot {
                let effective = pair.picked.get_untracked().or(default);
                if effective.is_none() {
                    continue;
                }
                if let Some(o) = pair.last_output.get_untracked() {
                    letters.extend(o.love_letters);
                }
            }
            for co_page in &co {
                letters.push(build_a_love_letter(
                    LoveLetterSketch::CreateNewPage {
                        page_id: co_page.page_id.clone(),
                        is_main_menu_page: co_page.is_main_menu_page,
                    },
                    co_page.client_blob.clone(),
                ));
            }
            let dv = dv.clone();
            let rv = rv.clone();
            let mf = mf.clone();
            let uid = uid.clone();
            let t = t.clone();
            spawn_local(async move {
                match finish_reconcile(dv, rv, letters, mf, uid, t).await {
                    Ok(()) => reload_the_page(),
                    Err(e) => {
                        show_popup(format!("reconcile failed: {e}"));
                        finishing_sig.set(false);
                    }
                }
            });
        })
    };

    view! {
        <div class="rp-meta">
            <span><strong>"drift"</strong>{target_drift_version}</span>
            <span><strong>"reconcile"</strong>{target_reconcile_version}</span>
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
            {move || {
                let n = resolved_count.get();
                if total == 0 {
                    None
                } else {
                    Some(view! {
                        <span class="rp-count-note">
                            {format!("{n} of {total} resolved")}
                        </span>
                    })
                }
            }}
        </div>

        {move || {
            let sel = selected.get();
            let pairs_now = pairs_signal.get();
            let default_now = default_picked.get();

            let Some(pid) = sel else {
                return view! {
                    <div class="rp-box">
                        <div class="rp-chooser">
                            <div class="rp-chooser-row">
                                {pairs_now
                                    .iter()
                                    .enumerate()
                                    .map(|(i, pair)| {
                                        let n = i + 1;
                                        let pid_select = pair.page.page_id.clone();
                                        let pid_sel = pair.page.page_id.clone();
                                        let pid_res = pair.page.page_id.clone();
                                        let is_res_initial = pair.picked.get_untracked().is_some()
                                            || default_now.is_some();
                                        view! {
                                            <button
                                                type="button"
                                                class=move || {
                                                    let is_sel = selected
                                                        .get()
                                                        .as_deref()
                                                        == Some(pid_sel.as_str());
                                                    let is_res = pairs_signal
                                                        .get()
                                                        .iter()
                                                        .find(|p| p.page.page_id == pid_res)
                                                        .map(|p| {
                                                            p.picked.get().is_some()
                                                                || default_picked.get().is_some()
                                                        })
                                                        .unwrap_or(is_res_initial);
                                                    match (is_sel, is_res) {
                                                        (true, true) => {
                                                            "rp-chooser-btn rp-chooser-btn-selected rp-chooser-btn-resolved"
                                                        }
                                                        (true, false) => {
                                                            "rp-chooser-btn rp-chooser-btn-selected"
                                                        }
                                                        (false, true) => {
                                                            "rp-chooser-btn rp-chooser-btn-resolved"
                                                        }
                                                        (false, false) => "rp-chooser-btn",
                                                    }
                                                }
                                                title=pid_select.clone()
                                                on:click=move |_| {
                                                    selected.set(Some(pid_select.clone()));
                                                    reviewing.set(None);
                                                }
                                            >
                                                {n}
                                            </button>
                                        }
                                    })
                                    .collect_view()}
                            </div>
                        </div>
                        <NothingSelectedPanel
                            default_picked=default_picked
                            on_set_default=on_set_default
                            ready_to_submit=ready_to_submit.get()
                            review=reviewing.get().map(|r| r.0)
                            review_client_only=reviewing
                                .get()
                                .map(|r| r.1)
                                .unwrap_or_default()
                            finishing=finishing.get()
                            on_submit_clicked=on_submit_clicked
                            on_review_back=on_review_back
                            on_send_clicked=on_send_clicked
                        />
                    </div>
                }
                    .into_any();
            };

            let Some((n, pair)) = pairs_now
                .iter()
                .enumerate()
                .find(|(_, p)| p.page.page_id == pid)
            else {
                return view! {
                    <div class="rp-box">
                        <div class="rp-fatal">"Page not found."</div>
                    </div>
                }
                    .into_any();
            };
            let number = n + 1;
            let effective = pair.picked.get().or(default_now);
            let pairs_sig = pairs_signal;

            let on_close = {
                let pid = pair.page.page_id.clone();
                Callback::new(move |_: ()| {
                    selected.set(None);
                    let _ = &pid;
                })
            };

            let Some(side) = effective else {
                let pid = pair.page.page_id.clone();
                let on_pick = Callback::new(move |s: PickedSide| {
                    let target = pid.clone();
                    pairs_sig.update(|ps| {
                        if let Some(p) =
                            ps.iter_mut().find(|p| p.page.page_id == target)
                        {
                            p.picked.set(Some(s));
                        }
                    });
                });
                return view! {
                    <PickWhichSideWinsScreen
                        number=number
                        page_id=pair.page.page_id.clone()
                        client_blocks=pair.page.client_blocks.clone()
                        server_blocks=pair.page.server_blocks.clone()
                        on_pick=on_pick
                        on_close=on_close
                    />
                }
                    .into_any();
            };

            let server_page = server_page_for_viewer(&pair.page);
            let client_page = client_page_for_viewer(&pair.page);
            let boss = pair.server_boss.clone();

            let Some(boss) = boss else {
                return view! {
                    <div class="rp-box">
                        <div class="rp-fatal">
                            "This page could not be decoded. Reload and retry."
                        </div>
                    </div>
                }
                    .into_any();
            };

            let pid_reset = pair.page.page_id.clone();
            let on_reset = Callback::new(move |_: ()| {
                let target = pid_reset.clone();
                pairs_sig.update(|ps| {
                    if let Some(p) =
                        ps.iter_mut().find(|p| p.page.page_id == target)
                    {
                        p.picked.set(None);
                        p.last_output.set(None);
                    }
                });
            });

            // Set `last_output` directly on the pair's own signal. Do
            // NOT go through `pairs_sig.update`: the outer view closure
            // reads `pairs_signal.get()`, so touching it re-runs the
            // closure and remounts InlineDiffViewerHost → DiffViewer
            // fresh, wiping the in-progress edit. Only `picked` and
            // `selected` are meant to change while the viewer is open
            // (they cause the remount into a different state on purpose).
            let on_done = {
                let last_output_sig = pair.last_output;
                Callback::new(move |out: ViewerOutput| {
                    last_output_sig.set(Some(out));
                })
            };

            let on_host_close = {
                let last_output_sig = pair.last_output;
                Callback::new(move |out: ViewerOutput| {
                    last_output_sig.set(Some(out));
                    selected.set(None);
                })
            };

            let show_reset = default_picked.get().is_none();

            view! {
                <InlineDiffViewerHost
                    number=number
                    page_id=pair.page.page_id.clone()
                    server_page=server_page
                    client_page=client_page
                    kept_side=side
                    server_page_boss=boss
                    show_reset=show_reset
                    on_reset=on_reset
                    on_done=on_done
                    on_close=on_host_close
                />
            }
                .into_any()
        }}
    }
    .into_any()
}