//! Caller pov: top-level layout. `pairs` holds every page pair and its
//! per-pair state (picked side, last viewer output). Done and X in the
//! viewer both save the last output; Done additionally shows the
//! sketches. Submit aggregates sketches across every picked pair.

use leptos::prelude::*;

use block_diff_viewer_leptos::{LoveLetter, ViewerOutput};

use crate::chooser::Chooser;
use crate::pick_screen::{PickScreen, PickedSide};
use crate::placeholder_data::{placeholder_items, PagePair};
use crate::viewer_host::ViewerHost;

#[component]
pub fn AppMain() -> impl IntoView {
    let pairs: RwSignal<Vec<PagePair>> = RwSignal::new(placeholder_items());
    let selected: RwSignal<Option<String>> = RwSignal::new(None);
    let default_picked: RwSignal<Option<PickedSide>> = RwSignal::new(None);
    let preview: RwSignal<Option<(String, Vec<String>)>> = RwSignal::new(None);
    let submit_result: RwSignal<Option<Vec<(String, Vec<String>)>>> = RwSignal::new(None);

    let selected_id = Memo::new(move |_| selected.get());

    let on_select = Callback::new(move |id: String| {
        selected.update(|s| {
            if s.as_deref() == Some(id.as_str()) {
                *s = None;
            } else {
                *s = Some(id);
            }
        });
        preview.set(None);
        submit_result.set(None);
    });

    view! {
        <div class="container">
            <div class="viewer-region">
                {move || {
                    let sel = selected_id.get();
                    let Some(pid) = sel else {
                        return view! {
                            <div class="nothing-selected">
                                <Chooser
                                    pairs=pairs
                                    selected=selected
                                    on_select=on_select
                                />
                                <NothingSelectedBody
                                    pairs=pairs
                                    default_picked=default_picked
                                    submit_result=submit_result
                                />
                            </div>
                        }
                            .into_any();
                    };
                    let pairs_snapshot = pairs.get();
                    let Some(pair_index) = pairs_snapshot.iter().position(|p| p.page_id == pid)
                    else {
                        return view! {
                            <div class="no-diff-open">
                                <h2>"Page not found"</h2>
                            </div>
                        }
                            .into_any();
                    };
                    let pair = pairs_snapshot[pair_index].clone();
                    let number = pair_index + 1;
                    let effective_picked = pair.picked.or(default_picked.get());

                    let Some(picked_side) = effective_picked else {
                        let pid_pick = pid.clone();
                        let on_pick = Callback::new(move |side: PickedSide| {
                            pairs.update(|ps| {
                                if let Some(p) =
                                    ps.iter_mut().find(|p| p.page_id == pid_pick)
                                {
                                    p.picked = Some(side);
                                }
                            });
                        });
                        let on_close = Callback::new(move |_: ()| {
                            selected.set(None);
                            preview.set(None);
                        });
                        return view! {
                            <PickScreen
                                number=number
                                page_id=pid
                                client_blocks=pair.client_page
                                server_blocks=pair.server_page
                                on_pick=on_pick
                                on_close=on_close
                            />
                        }
                            .into_any();
                    };

                    let server_for_viewer = pair.server_page_for_viewer();
                    let client_for_viewer = pair.client_page_for_viewer();
                    let kept_side_for_viewer = match picked_side {
                        PickedSide::Mine => {
                            block_diff_viewer_leptos::PickedSide::Mine
                        }
                        PickedSide::Server => {
                            block_diff_viewer_leptos::PickedSide::Server
                        }
                    };

                    let pid_done = pid.clone();
                    let on_done = Callback::new(move |out: ViewerOutput| {
                        let summaries: Vec<String> =
                            out.love_letters.iter().map(summary).collect();
                        let pid_save = pid_done.clone();
                        pairs.update(|ps| {
                            if let Some(p) =
                                ps.iter_mut().find(|p| p.page_id == pid_save)
                            {
                                p.last_output = Some(out.clone());
                            }
                        });
                        preview.set(Some((pid_done.clone(), summaries)));
                    });

                    let pid_close = pid.clone();
                    let on_close = Callback::new(move |out: ViewerOutput| {
                        let pid_save = pid_close.clone();
                        pairs.update(|ps| {
                            if let Some(p) =
                                ps.iter_mut().find(|p| p.page_id == pid_save)
                            {
                                p.last_output = Some(out);
                            }
                        });
                        selected.set(None);
                        preview.set(None);
                    });

                    let pid_reset = pid.clone();
                    let on_reset = Callback::new(move |_: ()| {
                        pairs.update(|ps| {
                            if let Some(p) =
                                ps.iter_mut().find(|p| p.page_id == pid_reset)
                            {
                                p.picked = None;
                                p.last_output = None;
                            }
                        });
                        preview.set(None);
                    });

                    let show_reset = default_picked.get().is_none();

                    let pid_for_preview = pid.clone();
                    let preview_for_page = Signal::derive(move || {
                        preview.get().and_then(|(id, v)| {
                            if id == pid_for_preview { Some(v) } else { None }
                        })
                    });

                    view! {
                        <ViewerHost
                            number=number
                            page_id=pid
                            server_page=Some(server_for_viewer)
                            client_page=client_for_viewer
                            kept_side=kept_side_for_viewer
                            show_reset=show_reset
                            preview=preview_for_page
                            on_reset=on_reset
                            on_done=on_done
                            on_close=on_close
                        />
                    }
                        .into_any()
                }}
            </div>
        </div>
    }
}

#[component]
fn NothingSelectedBody(
    pairs: RwSignal<Vec<PagePair>>,
    default_picked: RwSignal<Option<PickedSide>>,
    submit_result: RwSignal<Option<Vec<(String, Vec<String>)>>>,
) -> impl IntoView {
    view! {
        <>
            <div class="default-picker">
                <span class="default-picker-label">"Default for everything:"</span>
                <button
                    class=move || {
                        if default_picked.get().is_none() {
                            "default-btn default-btn-active"
                        } else {
                            "default-btn"
                        }
                    }
                    on:click=move |_| default_picked.set(None)
                >
                    "Ask each time"
                </button>
                <button
                    class=move || {
                        if default_picked.get() == Some(PickedSide::Mine) {
                            "default-btn default-btn-active"
                        } else {
                            "default-btn"
                        }
                    }
                    on:click=move |_| default_picked.set(Some(PickedSide::Mine))
                >
                    "Always mine"
                </button>
                <button
                    class=move || {
                        if default_picked.get() == Some(PickedSide::Server) {
                            "default-btn default-btn-active"
                        } else {
                            "default-btn"
                        }
                    }
                    on:click=move |_| default_picked.set(Some(PickedSide::Server))
                >
                    "Always server's"
                </button>
            </div>
            <div class="nothing-selected-body">
                {move || {
                    if let Some(result) = submit_result.get() {
                        return view! {
                            <div class="sketch-panel">
                                <h2>"Submitted — love letters that would be sent:"</h2>
                                {if result.is_empty() {
                                    view! {
                                        <div class="no-diff-open">
                                            <p>"(no love letters — every picked page matches the server)"</p>
                                        </div>
                                    }
                                        .into_any()
                                } else {
                                    result
                                        .into_iter()
                                        .map(|(page_id, list)| {
                                            view! {
                                                <div class="sketch-page">
                                                    <div class="sketch-page-id">{page_id}</div>
                                                    <ul>
                                                        {list
                                                            .into_iter()
                                                            .map(|l| view! { <li>{l}</li> })
                                                            .collect_view()}
                                                    </ul>
                                                </div>
                                            }
                                        })
                                        .collect_view()
                                        .into_any()
                                }}
                            </div>
                        }
                            .into_any();
                    }
                    let current_pairs = pairs.get();
                    let any_picked = current_pairs.iter().any(|p| p.picked.is_some());
                    let pairs_for_submit = pairs;
                    let submit_state = submit_result;
                    let on_submit = Callback::new(move |_: ()| {
                        let snapshot = pairs_for_submit.get_untracked();
                        let mut result = Vec::new();
                        for pair in snapshot {
                            if pair.picked.is_none() {
                                continue;
                            }
                            let Some(out) = pair.last_output else {
                                continue;
                            };
                            let summaries: Vec<String> =
                                out.love_letters.iter().map(summary).collect();
                            if !summaries.is_empty() {
                                result.push((pair.page_id, summaries));
                            }
                        }
                        submit_state.set(Some(result));
                    });
                    if !any_picked {
                        view! {
                            <div class="no-diff-open">
                                <h2>"No diff open"</h2>
                                <p>"Click a numbered page above to open its diff."</p>
                            </div>
                        }
                            .into_any()
                    } else {
                        view! {
                            <div class="no-diff-open">
                                <h2>"Ready to submit"</h2>
                                <p>"All picked pages will be turned into love letters."</p>
                                <button class="submit-btn" on:click=move |_| on_submit.run(())>
                                    "Submit"
                                </button>
                            </div>
                        }
                            .into_any()
                    }
                }}
            </div>
        </>
    }
}

fn summary(l: &LoveLetter) -> String {
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