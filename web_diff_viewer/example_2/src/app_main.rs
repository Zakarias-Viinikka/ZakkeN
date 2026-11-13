//! Caller pov: top-level layout. `pairs` holds every page pair and its
//! per-pair state (picked side, last saved edited blocks). Done in the
//! viewer computes a preview for that page and stores it in `preview`;
//! the viewer reads it and shows it below the diff. X saves edits and
//! closes. Submit (in nothing-selected) aggregates love letters for
//! every picked page and shows them.

use leptos::prelude::*;

use generate_love_letters::{generate_sketches, Block, LoveLetterSketch};

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

                    let (left_texts, right_texts) = match (
                        picked_side,
                        pair.edited_kept_blocks.as_ref(),
                    ) {
                        (PickedSide::Mine, Some(edited)) => (
                            pair.server_page.iter().map(|b| b.text.clone()).collect(),
                            edited.iter().map(|b| b.text.clone()).collect(),
                        ),
                        (PickedSide::Mine, None) => (
                            pair.server_page.iter().map(|b| b.text.clone()).collect(),
                            pair.client_page.iter().map(|b| b.text.clone()).collect(),
                        ),
                        (PickedSide::Server, Some(edited)) => (
                            pair.client_page.iter().map(|b| b.text.clone()).collect(),
                            edited.iter().map(|b| b.text.clone()).collect(),
                        ),
                        (PickedSide::Server, None) => (
                            pair.client_page.iter().map(|b| b.text.clone()).collect(),
                            pair.server_page.iter().map(|b| b.text.clone()).collect(),
                        ),
                    };

                    let kept_template: Vec<Block> = match picked_side {
                        PickedSide::Mine => pair.client_page.clone(),
                        PickedSide::Server => pair.server_page.clone(),
                    };
                    let server_page = pair.server_page.clone();
                    let pid_save = pid.clone();

                    let on_done = {
                        let kept_template = kept_template.clone();
                        let server_page = server_page.clone();
                        let pid_save = pid_save.clone();
                        let pid_for_preview = pid.clone();
                        Callback::new(move |blocks: Vec<String>| {
                            let user_blocks: Vec<Block> = kept_template
                                .iter()
                                .zip(blocks.iter())
                                .map(|(orig, text)| Block {
                                    block_id: orig.block_id.clone(),
                                    text: text.clone(),
                                    metadata: orig.metadata.clone(),
                                    frac_position: orig.frac_position.clone(),
                                })
                                .collect();
                            let summaries = match generate_sketches(
                                &pid_save,
                                &server_page,
                                &user_blocks,
                            ) {
                                Ok(sketches) => sketches.iter().map(summary).collect(),
                                Err(e) => {
                                    leptos::logging::log!(
                                        "generate_sketches failed: {e:?}"
                                    );
                                    Vec::new()
                                }
                            };
                            let pid_save2 = pid_save.clone();
                            pairs.update(|ps| {
                                if let Some(p) =
                                    ps.iter_mut().find(|p| p.page_id == pid_save2)
                                {
                                    p.edited_kept_blocks = Some(user_blocks);
                                }
                            });
                            preview.set(Some((pid_for_preview.clone(), summaries)));
                        })
                    };

                    let on_close = {
                        let kept_template = kept_template.clone();
                        Callback::new(move |blocks: Vec<String>| {
                            let user_blocks: Vec<Block> = kept_template
                                .iter()
                                .zip(blocks.iter())
                                .map(|(orig, text)| Block {
                                    block_id: orig.block_id.clone(),
                                    text: text.clone(),
                                    metadata: orig.metadata.clone(),
                                    frac_position: orig.frac_position.clone(),
                                })
                                .collect();
                            let pid_save = pid_save.clone();
                            pairs.update(|ps| {
                                if let Some(p) =
                                    ps.iter_mut().find(|p| p.page_id == pid_save)
                                {
                                    p.edited_kept_blocks = Some(user_blocks);
                                }
                            });
                            selected.set(None);
                            preview.set(None);
                        })
                    };

                    let pid_reset = pid.clone();
                    let on_reset = Callback::new(move |_: ()| {
                        pairs.update(|ps| {
                            if let Some(p) =
                                ps.iter_mut().find(|p| p.page_id == pid_reset)
                            {
                                p.picked = None;
                                p.edited_kept_blocks = None;
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
                            left=left_texts
                            right=right_texts
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
                    let current_pairs = pairs.get();
                    let submit_clicked = submit_result.get();
                    if let Some(result) = submit_clicked {
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
                    let any_picked = current_pairs.iter().any(|p| p.picked.is_some());
                    let pairs_for_submit = pairs;
                    let server_pages: Vec<(String, Vec<Block>)> = current_pairs
                        .iter()
                        .filter(|p| p.picked.is_some())
                        .map(|p| (p.page_id.clone(), p.server_page.clone()))
                        .collect();
                    let submit_state = submit_result;
                    let on_submit = Callback::new(move |_: ()| {
                        let mut result = Vec::new();
                        let snapshot = pairs_for_submit.get_untracked();
                        for pair in snapshot {
                            let Some(_) = pair.picked else { continue };
                            let kept = pair
                                .edited_kept_blocks
                                .clone()
                                .unwrap_or_else(|| match pair.picked.unwrap() {
                                    PickedSide::Mine => pair.client_page.clone(),
                                    PickedSide::Server => pair.server_page.clone(),
                                });
                            match generate_sketches(
                                &pair.page_id,
                                &pair.server_page,
                                &kept,
                            ) {
                                Ok(sketches) => {
                                    let summaries: Vec<String> =
                                        sketches.iter().map(summary).collect();
                                    if !summaries.is_empty() {
                                        result.push((pair.page_id, summaries));
                                    }
                                }
                                Err(e) => {
                                    leptos::logging::log!(
                                        "generate_sketches failed: {e:?}"
                                    );
                                }
                            }
                        }
                        let _ = server_pages;
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

fn summary(s: &LoveLetterSketch) -> String {
    match s {
        LoveLetterSketch::EditBlock { block_id, .. } => format!("EditBlock({block_id})"),
        LoveLetterSketch::RemoveBlock { block_id, .. } => format!("RemoveBlock({block_id})"),
        LoveLetterSketch::CreateNewBlock { .. } => "CreateNewBlock".to_string(),
        LoveLetterSketch::CreateNewBlockWithId { block_id, .. } => {
            format!("CreateNewBlockWithId({block_id})")
        }
        LoveLetterSketch::MoveBlock { block_id, .. } => format!("MoveBlock({block_id})"),
        LoveLetterSketch::DisablePage { page_id } => format!("DisablePage({page_id})"),
        LoveLetterSketch::CreateNewPage { page_id, .. } => format!("CreateNewPage({page_id})"),
    }
}