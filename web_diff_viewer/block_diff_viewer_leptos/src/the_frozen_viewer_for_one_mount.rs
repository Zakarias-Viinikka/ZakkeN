//! Caller pov: the inner half of the viewer. Not called directly — the
//! outer DiffViewer mounts one of these per input snapshot.
//!
//! Internals pov: computes the diff once, builds the frozen row list,
//! creates the visual order signal AND the per-row edit-state signals,
//! renders the grid, and wires the drag.
//!
//! The per-row state signals are created once, before the view, in a
//! Vec indexed by row id. They are NOT created inside the <For> child
//! closure. That way a reorder permutes rows without changing ids, so
//! state survives a drag. See coding_style.md on shared_owner.
//!
//! Drag flow (see also shared_owner::drag_in_progress_feedback and
//! the_drag_session_state, whose headers document their own touchers):
//! - Handle mousedown calls start_drag with (row_id, pointer_y). Reads
//!   all right-cell rects in content coordinates and stores a
//!   non-reactive DragSession in a StoredValue.
//! - Window mousemove: until the pointer moves more than
//!   DRAG_THRESHOLD_Y on the y axis (and the session isn't "moved"
//!   yet), does nothing. Once it has moved, sets the drag-feedback
//!   signals. Every event also triggers autoscroll when the pointer is
//!   near the scroll body's top or bottom edge.
//! - Window mouseup: if the drag moved and the final slot differs
//!   from the current position, builds a RowMoveEvent and applies it
//!   to visual_row_order. Then clears the session and the feedback
//!   signals. Also clears on Escape or window blur, reporting nothing
//!   (cancel, not commit).
//!
//! Autoscroll is intentionally driven by mousemove and not by a timer:
//! a stationary pointer does not autoscroll. A wheel works normally
//! during the drag because we only call scroll_by on mousemove.

use std::ops::Range;
use std::sync::Arc;

use block_text_diff::{
    inline_spans, rows_for_two_pages, LeftMarker, Relationship, RightMarker,
};
use leptos::ev;
use leptos::html::{Div, Textarea};
use leptos::prelude::*;
use web_sys::wasm_bindgen::JsCast;
use web_sys::wasm_bindgen::JsValue;

use crate::finish_handle::FinishHandle;
use crate::map_caret_across_edit::map_caret_across_edit;
use crate::reorder_rows_by_drag_math::{
    apply_row_move_event, build_row_move_event,
    insertion_slot_for_dragged_center,
};
use crate::segs_for_highlighting::segs_for_a_line;
use crate::complex_state::drag_in_progress_feedback::{
    DragInProgressFeedback, DropEdge, DropIndicator,
};
use crate::complex_state::visual_row_order::create_visual_row_order;
use crate::the_drag_session_state::{
    auto_scroll_if_pointer_is_near_the_edge, read_pointer_y,
    read_row_rects_in_content_coords, read_scroll_top, DragSession,
};
use crate::the_edit_state_of_one_block_of_text::BlockEditState;
use crate::the_pure_function_that_builds_the_output::output_right_side_texts_in_visual_order;

/// Minimum vertical movement in CSS pixels before a mousedown counts as
/// a drag rather than a click. Below this the session stays "not moved"
/// and mouseup is a no-op.
const DRAG_THRESHOLD_Y: f32 = 4.0;

#[component]
pub(crate) fn TheFrozenViewerForOneMount(
    left: Vec<String>,
    right: Vec<String>,
    output: WriteSignal<Vec<String>>,
    handle: FinishHandle,
    on_finished: Option<Callback<()>>,
) -> impl IntoView {
    let left_arc = Arc::new(left);
    let right_arc = Arc::new(right);
    let rows = Arc::new(rows_for_two_pages(left_arc.as_ref(), right_arc.as_ref()));
    let visual_order = create_visual_row_order((0..rows.len()).collect());
    let scroll_body_ref = NodeRef::<Div>::new();
    let drag_feedback = DragInProgressFeedback::new();
    let drag_session: StoredValue<Option<DragSession>, LocalStorage> =
        StoredValue::new_local(None);

    let show_original: RwSignal<bool> = RwSignal::new(false);
    let effective_order: Memo<Vec<usize>> = {
        let rows_for_order = Arc::clone(&rows);
        Memo::new(move |_| {
            if show_original.get() {
                (0..rows_for_order.len()).collect()
            } else {
                visual_order.get()
            }
        })
    };

    let states: Arc<Vec<Option<RwSignal<BlockEditState>>>> = {
        let rows_for_states = Arc::clone(&rows);
        let left_for_states = Arc::clone(&left_arc);
        let right_for_states = Arc::clone(&right_arc);
        let v: Vec<Option<RwSignal<BlockEditState>>> = (0..rows_for_states.len())
            .map(|i| {
                let row = rows_for_states.get(i)?;
                let right_slot = row.right?;
                let right_text = right_for_states
                    .get(right_slot.block_index)
                    .cloned()
                    .unwrap_or_default();
                let left_text = row
                    .left
                    .and_then(|s| left_for_states.get(s.block_index).cloned());
                let initial_highlights: Vec<Range<usize>> =
                    if row.relationship == Relationship::Edit {
                        match left_text.as_deref() {
                            Some(l) => inline_spans(l, &right_text).1,
                            None => Vec::new(),
                        }
                    } else {
                        Vec::new()
                    };
                Some(RwSignal::new(BlockEditState::with_highlights(
                    right_text,
                    initial_highlights,
                )))
            })
            .collect();
        Arc::new(v)
    };

    let edited_count: Memo<usize> = {
        let states_for_count = Arc::clone(&states);
        Memo::new(move |_| {
            states_for_count
                .iter()
                .filter(|s| {
                    s.as_ref()
                        .map(|sig| sig.get().is_edited())
                        .unwrap_or(false)
                })
                .count()
        })
    };

    let live_right_number_by_row = {
        let rows_for_numbers = Arc::clone(&rows);
        Memo::new(move |_| {
            let order = effective_order.get();
            let mut numbers: Vec<Option<usize>> = vec![None; rows_for_numbers.len()];
            let mut count = 0usize;
            for &id in &order {
                if let Some(row) = rows_for_numbers.get(id) {
                    if row.right.is_some() {
                        count += 1;
                        numbers[id] = Some(count);
                    }
                }
            }
            numbers
        })
    };

    Effect::new(move |_| {
        output.set(Vec::new());
    });

    let last_counter = StoredValue::new(handle.counter.get_untracked());
    Effect::new({
        let rows = Arc::clone(&rows);
        let states = Arc::clone(&states);
        move |_| {
            let current = handle.counter.get();
            if current == last_counter.get_value() {
                return;
            }
            last_counter.set_value(current);
            let order = visual_order.get_untracked();
            let texts = output_right_side_texts_in_visual_order(&rows, &order, |row_id| {
                states
                    .get(row_id)
                    .and_then(|o| *o)
                    .map(|sig| sig.get_untracked().current().to_string())
            });
            output.set(texts);
            if let Some(cb) = on_finished {
                cb.run(());
            }
        }
    });

    // ----- drag wiring -----

    let start_drag = {
        let scroll_body_ref = scroll_body_ref;
        let drag_session = drag_session;
        move |row_id: usize, pointer_y: f32| {
            let Some(scroll_el) = scroll_body_ref.get_untracked() else {
                return;
            };
            let scroll_html: &web_sys::HtmlElement = scroll_el.as_ref();
            let scroll_top = read_scroll_top(scroll_html);
            let Ok(rects) = read_row_rects_in_content_coords(scroll_html) else {
                return;
            };
            drag_session.set_value(Some(DragSession {
                dragged_row_id: row_id,
                start_pointer_y: pointer_y,
                start_scroll_top: scroll_top,
                last_pointer_y: pointer_y,
                moved: false,
                row_rects: rects,
            }));
        }
    };

    let on_mousemove = {
        let scroll_body_ref = scroll_body_ref;
        let drag_session = drag_session;
        let drag_feedback = drag_feedback;
        move |e: ev::MouseEvent| {
            let Some(mut session) = drag_session.get_value() else {
                return;
            };
            let pointer_y = read_pointer_y(&e);
            session.last_pointer_y = pointer_y;

            let Some(scroll_el) = scroll_body_ref.get_untracked() else {
                return;
            };
            let scroll_html: &web_sys::HtmlElement = scroll_el.as_ref();
            let scroll_top_now = read_scroll_top(scroll_html);

            let dy = (pointer_y - session.start_pointer_y)
                + (scroll_top_now - session.start_scroll_top);

            if !session.moved && dy.abs() < DRAG_THRESHOLD_Y {
                drag_session.set_value(Some(session));
                return;
            }
            if !session.moved {
                session.moved = true;
                drag_feedback.dragged_row_id.set(Some(session.dragged_row_id));
            }
            drag_feedback.dragged_translate_y.set(dy);
            drag_session.set_value(Some(session.clone()));

            auto_scroll_if_pointer_is_near_the_edge(scroll_html, pointer_y);

            // Slot from midpoint of the dragged row's original rect,
            // shifted by dy.
            let Some(dragged_rect) = session
                .row_rects
                .iter()
                .find(|r| r.row_id == session.dragged_row_id)
            else {
                return;
            };
            let dragged_center = dragged_rect.midpoint() + dy;
            let others: Vec<(usize, f32)> = session
                .row_rects
                .iter()
                .filter(|r| r.row_id != session.dragged_row_id)
                .map(|r| (r.row_id, r.midpoint()))
                .collect();
            let slot = insertion_slot_for_dragged_center(&others, dragged_center);
            let current_slot = session
                .row_rects
                .iter()
                .take_while(|r| r.row_id != session.dragged_row_id)
                .count();
            if slot == current_slot {
                drag_feedback.drop_indicator.set(None);
            } else if slot < others.len() {
                drag_feedback.drop_indicator.set(Some(DropIndicator {
                    target_row_id: others[slot].0,
                    edge: DropEdge::Before,
                }));
            } else if let Some(last) = others.last() {
                drag_feedback.drop_indicator.set(Some(DropIndicator {
                    target_row_id: last.0,
                    edge: DropEdge::After,
                }));
            } else {
                drag_feedback.drop_indicator.set(None);
            }
        }
    };

    let on_mouseup = {
        let scroll_body_ref = scroll_body_ref;
        let drag_session = drag_session;
        let drag_feedback = drag_feedback;
        move |_e: ev::MouseEvent| {
            let Some(session) = drag_session.get_value() else {
                return;
            };
            let was_moved = session.moved;
            let dragged_row_id = session.dragged_row_id;

            let scroll_top_now = scroll_body_ref
                .get_untracked()
                .map(|el| {
                    let scroll_html: &web_sys::HtmlElement = el.as_ref();
                    read_scroll_top(scroll_html)
                })
                .unwrap_or(session.start_scroll_top);

            let final_slot = if was_moved {
                let dy = (session.last_pointer_y - session.start_pointer_y)
                    + (scroll_top_now - session.start_scroll_top);
                session
                    .row_rects
                    .iter()
                    .find(|r| r.row_id == dragged_row_id)
                    .map(|dragged_rect| {
                        let dragged_center = dragged_rect.midpoint() + dy;
                        let others: Vec<(usize, f32)> = session
                            .row_rects
                            .iter()
                            .filter(|r| r.row_id != dragged_row_id)
                            .map(|r| (r.row_id, r.midpoint()))
                            .collect();
                        insertion_slot_for_dragged_center(&others, dragged_center)
                    })
            } else {
                None
            };

            drag_feedback.clear();
            drag_session.set_value(None);

            if let Some(slot) = final_slot {
                let current_order = visual_order.get_untracked();
                if let Some(event) =
                    build_row_move_event(&current_order, dragged_row_id, slot)
                {
                    let new_order = apply_row_move_event(&current_order, &event);
                    visual_order.set(new_order);
                }
            }
        }
    };

    let on_escape_or_blur = {
        let drag_session = drag_session;
        let drag_feedback = drag_feedback;
        move || {
            if drag_session.get_value().is_some() {
                drag_feedback.clear();
                drag_session.set_value(None);
            }
        }
    };

    let mousemove_handle = window_event_listener(ev::mousemove, on_mousemove);
    let mouseup_handle = window_event_listener(ev::mouseup, on_mouseup);
    let keydown_handle = window_event_listener(ev::keydown, move |e: ev::KeyboardEvent| {
        if e.key() == "Escape" {
            on_escape_or_blur();
        }
    });
    let blur_handle = window_event_listener(ev::blur, move |_| on_escape_or_blur());
    on_cleanup(move || {
        mousemove_handle.remove();
        mouseup_handle.remove();
        keydown_handle.remove();
        blur_handle.remove();
    });

    let root_class = move || {
        if drag_feedback.dragged_row_id.get().is_some() {
            "dv-root dv-dragging"
        } else {
            "dv-root"
        }
    };

    let left_for_fallback = Arc::clone(&left_arc);
    let right_for_fallback = Arc::clone(&right_arc);

    view! {
        <div class=root_class>
            <div class="dv-toolbar">
                <div class="dv-legend">
                    <span class="dv-legend-item">
                        <span class="dv-legend-dot dv-legend-dot-delete"></span>
                        "deleted"
                    </span>
                    <span class="dv-legend-item">
                        <span class="dv-legend-dot dv-legend-dot-insert"></span>
                        "inserted"
                    </span>
                    <span class="dv-legend-item">
                        <span class="dv-legend-dot dv-legend-dot-edit"></span>
                        "edited"
                    </span>
                    <span class="dv-legend-item">
                        <span class="dv-legend-dot dv-legend-dot-move"></span>
                        "moved"
                    </span>
                    <span class="dv-legend-item">
                        <span class="dv-legend-dot dv-legend-dot-yours"></span>
                        "edited by you"
                    </span>
                </div>
                <span class="dv-edited-count">
                    {move || {
                        let n = edited_count.get();
                        if n == 0 {
                            String::new()
                        } else {
                            format!(
                                "{} {} edited by you",
                                n,
                                if n == 1 { "block" } else { "blocks" },
                            )
                        }
                    }}
                </span>
            </div>
            <div class="dv-scroll-body" node_ref=scroll_body_ref>
                <Show
                    when=move || !show_original.get()
                    fallback=move || view! {
                        <div class="dv-grid dv-grid-original">
                            <div class="dv-cell dv-cell-header dv-cell-left">
                                <span>"Before"</span>
                                <button
                                    type="button"
                                    class="dv-header-toggle"
                                    on:click=move |_| show_original.update(|v| *v = !*v)
                                >
                                    "Show current"
                                </button>
                            </div>
                            <div class="dv-cell dv-cell-header dv-cell-right">
                                <span>"After"</span>
                                <button
                                    type="button"
                                    class="dv-header-toggle"
                                    on:click=move |_| show_original.update(|v| *v = !*v)
                                >
                                    "Show current"
                                </button>
                            </div>
                            <div class="dv-original-pane dv-original-pane-left">
                                {left_for_fallback.iter().cloned().map(|text| view! {
                                    <div class="dv-original-block">{text}</div>
                                }).collect_view()}
                            </div>
                            <div class="dv-original-pane dv-original-pane-right">
                                {right_for_fallback.iter().cloned().map(|text| view! {
                                    <div class="dv-original-block">{text}</div>
                                }).collect_view()}
                            </div>
                        </div>
                    }
                >
                <div class="dv-grid">
                    <div class="dv-cell dv-cell-header dv-cell-left">
                        <span>"Before"</span>
                        <button
                            type="button"
                            class="dv-header-toggle"
                            on:click=move |_| show_original.update(|v| *v = !*v)
                        >
                            {move || if show_original.get() { "Show current" } else { "Show original" }}
                        </button>
                    </div>
                    <div class="dv-cell dv-cell-header dv-cell-right">
                        <span>"After — editable"</span>
                        <button
                            type="button"
                            class="dv-header-toggle"
                            on:click=move |_| show_original.update(|v| *v = !*v)
                        >
                            {move || if show_original.get() { "Show current" } else { "Show original" }}
                        </button>
                    </div>
                    <For
                        each=move || effective_order.get()
                        key=|id: &usize| *id
                        children={
                            let rows = Arc::clone(&rows);
                            let left_arc = Arc::clone(&left_arc);
                            let right_arc = Arc::clone(&right_arc);
                            let states = Arc::clone(&states);
                            move |row_id: usize| {
                                let rows_in = Arc::clone(&rows);
                                let left_in = Arc::clone(&left_arc);
                                let right_in = Arc::clone(&right_arc);
                                let states_in = Arc::clone(&states);
                                let row = rows_in
                                    .get(row_id)
                                    .cloned()
                                    .expect("row id out of range");
                                let left_text = row
                                    .left
                                    .and_then(|slot| left_in.get(slot.block_index).cloned());
                                let right_text = row
                                    .right
                                    .and_then(|slot| right_in.get(slot.block_index).cloned());
                                let left_number = row.left.map(|slot| slot.block_index + 1);
                                let data_kind = relationship_to_data_kind(row.relationship);
                                let left_bar_class = bar_class_for_left_marker(row.left_marker);
                                let right_bar_class = bar_class_for_right_marker(row.right_marker);
                                let left_empty = left_text.is_none();
                                let right_empty = right_text.is_none();
                                let has_right = !right_empty;

                                let left_highlights: Vec<Range<usize>> =
                                    if row.relationship == Relationship::Edit {
                                        match (left_text.as_deref(), right_text.as_deref()) {
                                            (Some(l), Some(r)) => inline_spans(l, r).0,
                                            _ => Vec::new(),
                                        }
                                    } else {
                                        Vec::new()
                                    };
                                let left_segments = renderable_segments(
                                    left_text.as_deref().unwrap_or(""),
                                    &left_highlights,
                                );
                                let left_content = render_segments_as_view(left_segments);

                                let state_for_cell: Option<RwSignal<BlockEditState>> =
                                    states_in.get(row_id).and_then(|o| *o);
                                let frozen_text_for_cell =
                                    right_text.clone().unwrap_or_default();
                                let frozen_highlights_for_cell: Vec<Range<usize>> =
                                    if row.relationship == Relationship::Edit && has_right {
                                        match left_text.as_deref() {
                                            Some(l) => inline_spans(l, &frozen_text_for_cell).1,
                                            None => Vec::new(),
                                        }
                                    } else {
                                        Vec::new()
                                    };
                                let right_cell_inner = move || -> AnyView {
                                    if !has_right {
                                        return view! { <div class="dv-content"></div> }
                                            .into_any();
                                    }
                                    if show_original.get() {
                                        let segs = renderable_segments(
                                            &frozen_text_for_cell,
                                            &frozen_highlights_for_cell,
                                        );
                                        return view! {
                                            <div class="dv-content">
                                                {render_segments_as_view(segs)}
                                            </div>
                                        }
                                        .into_any();
                                    }
                                    let state_signal = state_for_cell
                                        .expect("has_right implies a state signal");
                                    let initial = state_signal
                                        .get_untracked()
                                        .current()
                                        .to_string();
                                    let textarea_ref = NodeRef::<Textarea>::new();
                                    let state_for_input = state_signal;
                                    let state_for_sync = state_signal;
                                    let sync_ref = textarea_ref;

                                    Effect::new(move |_| {
                                        let new_value =
                                            state_for_sync.get().current().to_string();
                                        let Some(el) = sync_ref.get_untracked() else {
                                            return;
                                        };
                                        let old_value = el.value();
                                        if old_value == new_value {
                                            return;
                                        }
                                        let is_focused = web_sys::window()
                                            .and_then(|w| w.document())
                                            .and_then(|d| d.active_element())
                                            .map(|active| {
                                                let a: &JsValue = active.as_ref();
                                                let e: &JsValue = el.as_ref();
                                                a == e
                                            })
                                            .unwrap_or(false);
                                        let caret_before =
                                            el.selection_start().ok().flatten().unwrap_or(0);
                                        el.set_value(&new_value);
                                        if is_focused {
                                            let new_caret = map_caret_across_edit(
                                                &old_value,
                                                &new_value,
                                                caret_before,
                                            );
                                            let _ = el
                                                .set_selection_range(new_caret, new_caret);
                                        }
                                    });

                                    let on_input = move |e: leptos::ev::Event| {
                                        let Some(t) = e.target() else {
                                            return;
                                        };
                                        let ta: web_sys::HtmlTextAreaElement =
                                            t.unchecked_into();
                                        let new_val = ta.value();
                                        state_for_input
                                            .update(|s| s.user_typed(new_val));
                                    };

                                    let state_for_mirror = state_signal;
                                    view! {
                                        <div class="dv-content-editable">
                                            <div class="dv-mirror" aria-hidden="true">
                                                {move || {
                                                    let state = state_for_mirror.get();
                                                    let current = state.current().to_string();
                                                    let highlights: Vec<Range<usize>> =
                                                        state.current_highlights().to_vec();
                                                    let segs = renderable_segments(
                                                        &current,
                                                        &highlights,
                                                    );
                                                    render_segments_as_view(segs)
                                                }}
                                            </div>
                                            <textarea
                                                node_ref=textarea_ref
                                                class="dv-textarea"
                                                prop:value=initial
                                                rows="1"
                                                spellcheck="false"
                                                autocomplete="off"
                                                on:input=on_input
                                            ></textarea>
                                        </div>
                                    }
                                    .into_any()
                                };

                                let right_actions_lane = if has_right {
                                    let state = states_in
                                        .get(row_id)
                                        .and_then(|o| *o)
                                        .expect("has_right implies a state signal");
                                    let class_now = move || {
                                        let s = state.get();
                                        if !show_original.get()
                                            && (s.is_edited() || s.can_undo_reset())
                                        {
                                            "dv-actions-lane dv-actions-active"
                                        } else {
                                            "dv-actions-lane"
                                        }
                                    };
                                    view! {
                                        <div class=class_now>
                                            <Show when=move || state.get().can_reset()>
                                                <button
                                                    type="button"
                                                    class="dv-action-btn dv-action-reset"
                                                    aria-label="Reset to original"
                                                    title="Reset to original"
                                                    on:mousedown=move |e| e.prevent_default()
                                                    on:click=move |_| state.update(|s| s.reset())
                                                >
                                                    <svg
                                                        width="14"
                                                        height="14"
                                                        viewBox="0 0 24 24"
                                                        fill="none"
                                                        stroke="currentColor"
                                                        stroke-width="2"
                                                        stroke-linecap="round"
                                                        stroke-linejoin="round"
                                                        aria-hidden="true"
                                                    >
                                                        <path d="M3 12a9 9 0 1 0 3-6.7"></path>
                                                        <path d="M3 4v5h5"></path>
                                                    </svg>
                                                </button>
                                            </Show>
                                            <Show when=move || state.get().can_undo_reset()>
                                                <button
                                                    type="button"
                                                    class="dv-action-btn dv-action-undo"
                                                    aria-label="Undo reset"
                                                    title="Undo reset"
                                                    on:mousedown=move |e| e.prevent_default()
                                                    on:click=move |_| state.update(|s| s.undo_reset())
                                                >
                                                    <svg
                                                        width="14"
                                                        height="14"
                                                        viewBox="0 0 24 24"
                                                        fill="none"
                                                        stroke="currentColor"
                                                        stroke-width="2"
                                                        stroke-linecap="round"
                                                        stroke-linejoin="round"
                                                        aria-hidden="true"
                                                    >
                                                        <path d="M21 12a9 9 0 1 1-3-6.7"></path>
                                                        <path d="M21 4v5h-5"></path>
                                                    </svg>
                                                </button>
                                            </Show>
                                        </div>
                                    }
                                    .into_any()
                                } else {
                                    view! { <div class="dv-actions-lane"></div> }.into_any()
                                };

                                let draw_hairline = {
                                    let rows_for_hairline = Arc::clone(&rows);
                                    move || -> bool {
                                        let Some(r) = rows_for_hairline.get(row_id) else {
                                            return false;
                                        };
                                        let Some(my_hunk) = r.hunk_index else {
                                            return false;
                                        };
                                        let order = effective_order.get();
                                        let Some(pos) =
                                            order.iter().position(|&id| id == row_id)
                                        else {
                                            return false;
                                        };
                                        if pos == 0 {
                                            return false;
                                        }
                                        match rows_for_hairline.get(order[pos - 1]) {
                                            Some(prev) => prev.hunk_index != Some(my_hunk),
                                            None => true,
                                        }
                                    }
                                };
                                let live_right_number = move || -> Option<usize> {
                                    live_right_number_by_row.get().get(row_id).copied().flatten()
                                };

                                let data_hairline = Signal::derive(move || {
                                    if draw_hairline() {
                                        "true".to_string()
                                    } else {
                                        "false".to_string()
                                    }
                                });
                                let data_empty_left = if left_empty {
                                    "true".to_string()
                                } else {
                                    "false".to_string()
                                };
                                let data_empty_right = if right_empty {
                                    "true".to_string()
                                } else {
                                    "false".to_string()
                                };


                                // Bar colour is reactive on the row's edit
                                // state: while edited-by-you, both bars are
                                // yellow. Reset to original reverts to the
                                // diff's original colour.
                                // Chips that name the partner row on a
                                // moved block. Two purple bars alone say
                                // "this moved" but not "to where" — the
                                // chips make the two halves obviously
                                // paired.
                                let left_move_chip: Option<AnyView> =
                                    match row.left_marker {
                                        LeftMarker::MovedAway {
                                            partner_right_block_index,
                                            identical,
                                        } => {
                                            let rows_chip = Arc::clone(&rows_in);
                                            let live_numbers = live_right_number_by_row;
                                            let partner = partner_right_block_index;
                                            Some(
                                                view! {
                                                    <span class="dv-move-chip">
                                                        {move || {
                                                            let _ = effective_order.get();
                                                            let nums = live_numbers.get();
                                                            let partner_row_id = rows_chip
                                                                .iter()
                                                                .position(|r| {
                                                                    r.right
                                                                        .map(|s| s.block_index)
                                                                        == Some(partner)
                                                                });
                                                            let n = partner_row_id
                                                                .and_then(|id| {
                                                                    nums.get(id).copied().flatten()
                                                                });
                                                            match n {
                                                                Some(n) => format!(
                                                                    "moved to #{}{}",
                                                                    n,
                                                                    if identical { "" } else { " (edited)" },
                                                                ),
                                                                None => format!(
                                                                    "moved{}",
                                                                    if identical { "" } else { " (edited)" },
                                                                ),
                                                            }
                                                        }}
                                                    </span>
                                                }
                                                .into_any(),
                                            )
                                        }
                                        _ => None,
                                    };

                                let right_move_chip: Option<AnyView> =
                                    match row.right_marker {
                                        RightMarker::MovedHere {
                                            partner_left_block_index,
                                            identical,
                                        } => {
                                            let n = partner_left_block_index + 1;
                                            let label = if identical {
                                                format!("moved from #{n}")
                                            } else {
                                                format!("moved from #{n} (edited)")
                                            };
                                            Some(
                                                view! {
                                                    <span class="dv-move-chip">{label}</span>
                                                }
                                                .into_any(),
                                            )
                                        }
                                        _ => None,
                                    };

                                let states_for_left_bar = Arc::clone(&states_in);
                                let states_for_right_bar = Arc::clone(&states_in);
                                let left_bar_class_static: &'static str = left_bar_class;
                                let right_bar_class_static: &'static str = right_bar_class;
                                let left_class_signal = move || -> String {
                                    let is_edited = !show_original.get()
                                        && states_for_left_bar
                                            .get(row_id)
                                            .and_then(|o| *o)
                                            .map(|s| s.get().is_edited())
                                            .unwrap_or(false);
                                    if is_edited {
                                        "dv-bar dv-bar-edited-by-you".to_string()
                                    } else if left_bar_class_static.is_empty() {
                                        "dv-bar".to_string()
                                    } else {
                                        format!("dv-bar {}", left_bar_class_static)
                                    }
                                };
                                let right_class_signal = move || -> String {
                                    let is_edited = !show_original.get()
                                        && states_for_right_bar
                                            .get(row_id)
                                            .and_then(|o| *o)
                                            .map(|s| s.get().is_edited())
                                            .unwrap_or(false);
                                    if is_edited {
                                        "dv-bar dv-bar-edited-by-you".to_string()
                                    } else if right_bar_class_static.is_empty() {
                                        "dv-bar".to_string()
                                    } else {
                                        format!("dv-bar {}", right_bar_class_static)
                                    }
                                };

                                // Drag visual: the dragged row's cells get a
                                // translateY and a lift class. Both cells read
                                // the same two signals. Only the dragged row
                                // reads translate_y, so only it re-renders on
                                // pointer move.
                                let drag_style_left = move || -> String {
                                    if drag_feedback.dragged_row_id.get() == Some(row_id) {
                                        format!(
                                            "transform: translateY({}px);",
                                            drag_feedback.dragged_translate_y.get()
                                        )
                                    } else {
                                        String::new()
                                    }
                                };
                                let drag_style_right = drag_style_left.clone();
                                let drag_class_left = move || -> &'static str {
                                    if drag_feedback.dragged_row_id.get() == Some(row_id) {
                                        " dv-dragging"
                                    } else {
                                        ""
                                    }
                                };
                                let drag_class_right = drag_class_left;

                                let data_drop_left = move || -> String {
                                    let Some(ind) = drag_feedback.drop_indicator.get() else {
                                        return String::new();
                                    };
                                    if ind.target_row_id != row_id {
                                        return String::new();
                                    }
                                    match ind.edge {
                                        DropEdge::Before => "before".to_string(),
                                        DropEdge::After => "after".to_string(),
                                    }
                                };
                                let data_drop_right = data_drop_left.clone();

                                view! {
                                    <div class="dv-row" data-row-id=row_id.to_string()>
                                        <div
                                            class=move || format!("dv-cell dv-cell-left{}", drag_class_left())
                                            style=drag_style_left
                                            data-row-id=row_id.to_string()
                                            data-kind=data_kind
                                            data-empty=data_empty_left
                                            data-hairline=data_hairline
                                            data-drop=data_drop_left
                                        >
                                            <div class="dv-gutter dv-gutter-left">
                                                {left_number.map(|n| view! { <span>{n}</span> })}
                                            </div>
                                            <div class=left_class_signal></div>
                                            <div class="dv-content">
                                                {left_content}
                                                {left_move_chip}
                                            </div>
                                        </div>
                                        <div
                                            class=move || format!("dv-cell dv-cell-right{}", drag_class_right())
                                            style=drag_style_right
                                            data-row-id=row_id.to_string()
                                            data-kind=data_kind
                                            data-empty=data_empty_right
                                            data-hairline=data_hairline
                                            data-drop=data_drop_right
                                        >
                                            <div class="dv-gutter dv-gutter-right">
                                                {move || (has_right && !show_original.get()).then(|| view! {
                                                    <button
                                                        type="button"
                                                        class="dv-handle"
                                                        aria-label="Drag to reorder"
                                                        title="Drag to reorder"
                                                        on:mousedown={
                                                            let start_drag = start_drag;
                                                            move |e: ev::MouseEvent| {
                                                                e.prevent_default();
                                                                start_drag(row_id, read_pointer_y(&e));
                                                            }
                                                        }
                                                    >
                                                        <svg
                                                            width="8"
                                                            height="14"
                                                            viewBox="0 0 8 14"
                                                            fill="currentColor"
                                                            aria-hidden="true"
                                                        >
                                                            <circle cx="2" cy="2" r="1"></circle>
                                                            <circle cx="6" cy="2" r="1"></circle>
                                                            <circle cx="2" cy="7" r="1"></circle>
                                                            <circle cx="6" cy="7" r="1"></circle>
                                                            <circle cx="2" cy="12" r="1"></circle>
                                                            <circle cx="6" cy="12" r="1"></circle>
                                                        </svg>
                                                    </button>
                                                })}
                                                {move || {
                                                    live_right_number()
                                                        .map(|n| view! { <span class="dv-number">{n}</span> })
                                                }}
                                            </div>
                                            <div class=right_class_signal></div>
                                            <div class="dv-edited-lane"></div>
                                            <div class="dv-content-wrapper">
                                                {right_cell_inner}
                                                {right_move_chip}
                                            </div>
                                            {right_actions_lane}
                                        </div>
                                    </div>
                                }
                            }
                        }
                    />
                    <div class="dv-cell dv-cell-footer dv-cell-left"></div>
                    <div class="dv-cell dv-cell-footer dv-cell-right"></div>
                </div>
                </Show>
            </div>
        </div>
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RenderableSegment {
    text: String,
    is_highlighted: bool,
}

fn renderable_segments(text: &str, highlights: &[Range<usize>]) -> Vec<RenderableSegment> {
    if text.is_empty() {
        return Vec::new();
    }
    let fits = highlights.iter().all(|h| {
        h.start <= h.end
            && h.end <= text.len()
            && text.is_char_boundary(h.start)
            && text.is_char_boundary(h.end)
    });
    if !fits {
        return vec![RenderableSegment {
            text: text.to_string(),
            is_highlighted: false,
        }];
    }
    let segs = segs_for_a_line(text.len(), highlights);
    if segs.is_empty() {
        return vec![RenderableSegment {
            text: text.to_string(),
            is_highlighted: false,
        }];
    }
    segs.into_iter()
        .map(|s| RenderableSegment {
            text: text[s.start..s.end].to_string(),
            is_highlighted: s.is_highlighted,
        })
        .collect()
}

fn render_segments_as_view(segs: Vec<RenderableSegment>) -> impl IntoView {
    segs.into_iter()
        .map(|seg| {
            let class = if seg.is_highlighted { "dv-hl" } else { "" };
            view! { <span class=class>{seg.text}</span> }
        })
        .collect_view()
}

fn relationship_to_data_kind(r: Relationship) -> &'static str {
    match r {
        Relationship::Equal => "equal",
        Relationship::Edit => "edit",
        Relationship::CollapsedDeleteAndInsert => "collapsed",
        Relationship::LeftOnly => "left-only",
        Relationship::RightOnly => "right-only",
        Relationship::Moved => "moved",
    }
}

fn bar_class_for_left_marker(m: LeftMarker) -> &'static str {
    match m {
        LeftMarker::Unchanged => "",
        LeftMarker::Deleted => "dv-bar-delete",
        LeftMarker::Edited => "dv-bar-edit",
        // A moved block on the left (its original position) shows the
        // purple strip via CSS on .dv-cell-left[data-kind="moved"]::after,
        // always. The bar column itself is only filled when the move
        // also edited the text — then the blue edit bar shows next to
        // the purple strip. A pure move (identical text) has no bar
        // column, just the purple strip.
        LeftMarker::MovedAway { identical: true, .. } => "",
        LeftMarker::MovedAway { identical: false, .. } => "dv-bar-edit",
    }
}

fn bar_class_for_right_marker(m: RightMarker) -> &'static str {
    match m {
        RightMarker::Unchanged => "",
        RightMarker::Inserted => "dv-bar-insert",
        RightMarker::Edited => "dv-bar-edit",
        // On the destination side, a moved block is not an insert —
        // the text existed all along, it just lives at a new address.
        // Bar column empty unless the move also edited the text.
        RightMarker::MovedHere { identical: true, .. } => "",
        RightMarker::MovedHere { identical: false, .. } => "dv-bar-edit",
    }
}