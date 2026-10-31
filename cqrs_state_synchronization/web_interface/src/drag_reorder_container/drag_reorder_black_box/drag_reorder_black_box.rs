use leptos::prelude::*;
use leptos::ev;
use leptos::html::Div;
use std::collections::HashMap;
use std::time::Duration;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use super::cascade::step;
use super::drag_state::{DragState, RectDragState};
use super::heights::apply_heights;
use super::layout::{
    stage_height, stage_width, page_x, page_panel_height, PAGE_W, PAGE_TOP,
    VACUUM_W, RECT_W, RECT_H, RECT_HOME, BLOCK_W,
};
use super::mouse::{handle_mouse_move, handle_mouse_up};
use super::rapier_world::RapierWorld;
use super::sync::{sync_from_props, SyncState};
use crate::shared_owners::BlockRow;
use super::types::{BlockRenderInput, MoveEvent, PageInput};

#[component]
pub fn DragReorderBlackBox(
    #[prop(into)] pages: Signal<Vec<PageInput>>,
    #[prop(into)] blocks: Signal<Vec<BlockRow>>,
    on_move: Callback<MoveEvent>,
    #[prop(into)] render_block: Callback<BlockRenderInput, AnyView>,
) -> impl IntoView {
    let boxes = RwSignal::new(Vec::<super::layout::Block>::new());
    let drag: RwSignal<Option<DragState>> = RwSignal::new(None);
    let rect_drag: RwSignal<Option<RectDragState>> = RwSignal::new(None);
    let stored: RwSignal<Option<usize>> = RwSignal::new(None);
    let next_id: RwSignal<usize> = RwSignal::new(1);
    let id_map: RwSignal<HashMap<String, usize>> = RwSignal::new(HashMap::new());
    let page_count: RwSignal<usize> = RwSignal::new(0);
    let current_page_ids: RwSignal<Vec<String>> = RwSignal::new(Vec::new());

    let rapier = RapierWorld::new();
    let stage_ref = NodeRef::<Div>::new();

    let observer_stored: StoredValue<Option<web_sys::ResizeObserver>, LocalStorage> =
        StoredValue::new_local(None);
    {
        let boxes_obs = boxes;
        let rapier_obs = rapier.clone();
        let page_count_obs = page_count;
        let cb = Closure::<dyn FnMut(js_sys::Array, web_sys::ResizeObserver)>::new(
            move |entries: js_sys::Array, _obs: web_sys::ResizeObserver| {
                let mut pairs: Vec<(usize, f32)> = Vec::new();
                for i in 0..entries.length() {
                    let entry_val = entries.get(i);
                    let entry: web_sys::ResizeObserverEntry = entry_val.unchecked_into();
                    let target = entry.target();
                    let html: web_sys::HtmlElement = target.unchecked_into();
                    let h = html.offset_height() as f32;
                    if h < 1.0 {
                        continue;
                    }
                    let id_attr = html.get_attribute("data-id").unwrap_or_default();
                    if let Ok(id) = id_attr.parse::<usize>() {
                        pairs.push((id, h));
                    }
                }
                if pairs.is_empty() {
                    return;
                }
                apply_heights(pairs, boxes_obs, &rapier_obs, page_count_obs);
            },
        );
        let observer = web_sys::ResizeObserver::new(cb.as_ref().unchecked_ref())
            .expect("ResizeObserver::new");
        cb.forget();
        observer_stored.set_value(Some(observer));
    }

    {
        let rapier_effect = rapier.clone();
        Effect::new(move || {
            let state = SyncState {
                boxes,
                id_map,
                page_count,
                current_page_ids,
                next_id,
                stored,
                drag,
                rect_drag,
                rapier: &rapier_effect,
            };
            sync_from_props(pages, blocks, state);
        });
    }

    let interval_handle = set_interval_with_handle(
        move || {
            let mut bs = boxes.get_untracked();
            if step(&mut bs) {
                boxes.set(bs);
            }
        },
        Duration::from_millis(16),
    )
    .expect("failed to set interval");

    let on_mouse_move = move |e: ev::MouseEvent| {
        handle_mouse_move(e, stage_ref, drag, rect_drag);
    };

    let rapier_up = rapier.clone();
    let on_mouse_up = move |e: ev::MouseEvent| {
        handle_mouse_up(
            e,
            stage_ref,
            drag,
            rect_drag,
            boxes,
            stored,
            page_count,
            current_page_ids,
            &rapier_up,
            on_move,
        );
    };

    let mousemove_handle = window_event_listener(ev::mousemove, on_mouse_move);
    let mouseup_handle = window_event_listener(ev::mouseup, on_mouse_up);

    on_cleanup(move || {
        interval_handle.clear();
        mousemove_handle.remove();
        mouseup_handle.remove();
        observer_stored.with_value(|opt| {
            if let Some(o) = opt {
                o.disconnect();
            }
        });
    });

    view! {
        <>
            <style>{include_str!("drag_reorder_black_box.css")}</style>
            <div
                node_ref=stage_ref
                class="ps-stage"
                style:width=move || format!("{}px", stage_width(page_count.get()))
                style:height=move || format!("{}px", stage_height(&boxes.get()))
                on:mouseleave=move |_| {
                    drag.set(None);
                    rect_drag.set(None);
                }
            >
                <For
                    each=move || current_page_ids.get()
                    key=|pid| pid.clone()
                    children=move |pid| {
                        let page_idx = current_page_ids
                            .get_untracked()
                            .iter()
                            .position(|p| p == &pid)
                            .unwrap_or(0);
                        view! {
                            <div
                                class="ps-page-panel"
                                style:left=format!("{}px", page_x(page_idx))
                                style:top=format!("{}px", PAGE_TOP)
                                style:width=format!("{}px", PAGE_W)
                                style:height=move || format!("{}px", page_panel_height(stage_height(&boxes.get())))
                            >
                                <span class="ps-page-title">{pid.to_uppercase()}</span>
                            </div>
                        }
                    }
                />
                <For
                    each=move || boxes.get()
                    key=|b| b.internal_id
                    children=move |b: super::layout::Block| {
                        let id = b.internal_id;
                        let node_ref = NodeRef::<Div>::new();
                        let observer_for_mount = observer_stored;

                        Effect::new(move |_| {
                            let Some(el) = node_ref.get() else { return; };
                            observer_for_mount.with_value(|opt| {
                                if let Some(obs) = opt {
                                    let _ = obs.observe(&el);
                                }
                            });
                        });

                        let class_now = move || {
                            let mut c = String::from("ps-box");
                            if let Some(cur) = boxes.get().iter().find(|x| x.internal_id == id) {
                                if cur.is_title { c.push_str(" ps-box-title"); }
                                if cur.ghost { c.push_str(" ps-ghost"); }
                                if !cur.measured { c.push_str(" ps-unmeasured"); }
                            }
                            if let Some(d) = drag.get() {
                                if d.id == id { c.push_str(" ps-dragging"); }
                            }
                            c
                        };

                        let left_now = move || {
                            if let Some(d) = drag.get() {
                                if d.id == id { return format!("{}px", d.current.0); }
                            }
                            let x = boxes.get().iter().find(|x| x.internal_id == id).map(|x| x.x).unwrap_or(0.0);
                            format!("{}px", x)
                        };

                        let top_now = move || {
                            if let Some(d) = drag.get() {
                                if d.id == id { return format!("{}px", d.current.1); }
                            }
                            let y = boxes.get().iter().find(|x| x.internal_id == id).map(|x| x.y).unwrap_or(0.0);
                            format!("{}px", y)
                        };

                        let handle_class = move || {
                            let is_title = boxes.get().iter().find(|x| x.internal_id == id).map(|x| x.is_title).unwrap_or(false);
                            if is_title { "ps-handle ps-handle-pinned" } else { "ps-handle" }
                        };

                        let block_render_input = BlockRenderInput {
                            block_id: b.block_id.to_string(),
                            content: b.content.clone(),
                            ghost: Signal::derive(move || {
                                boxes.get().iter().find(|x| x.internal_id == id).map(|x| x.ghost).unwrap_or(false)
                            }),
                            is_title: b.is_title,
                        };

                        view! {
                            <div
                                node_ref=node_ref
                                class=class_now
                                style:left=left_now
                                style:top=top_now
                                style:width=format!("{}px", BLOCK_W)
                                data-id=id
                            >
                                <div class=handle_class on:mousedown=move |e| {
                                    if boxes.get_untracked().iter().find(|x| x.internal_id == id).map(|x| x.is_title).unwrap_or(false) {
                                        return;
                                    }
                                    if let Some(stage) = stage_ref.get() {
                                        let rect = stage.get_bounding_client_rect();
                                        let mx = e.client_x() as f32 - rect.left() as f32;
                                        let my = e.client_y() as f32 - rect.top() as f32;
                                        let live = boxes.get_untracked().iter().find(|x| x.internal_id == id).map(|x| (x.x, x.y)).unwrap_or((0.0, 0.0));
                                        drag.set(Some(DragState {
                                            id,
                                            offset: (mx - live.0, my - live.1),
                                            current: live,
                                            moved: false,
                                            start: (mx, my),
                                        }));
                                        e.prevent_default();
                                    }
                                }>"::"</div>
                                {render_block.run(block_render_input)}
                            </div>
                        }
                    }
                />
                <div class="ps-vacuum" style:left="0" style:top="0" style:width=format!("{}px", VACUUM_W) style:height="100%"></div>
                {move || stored.get().map(|s_id| {
                    let content_val = boxes.get().iter().find(|b| b.internal_id == s_id).map(|b| b.content.get()).unwrap_or_default();
                    let label: String = content_val.chars().take(10).collect();
                    view! {
                        <div
                            class="ps-vacuum-rect"
                            style:left=move || format!("{}px", if let Some(r) = rect_drag.get() { r.current.0 } else { RECT_HOME.0 })
                            style:top=move || format!("{}px", if let Some(r) = rect_drag.get() { r.current.1 } else { RECT_HOME.1 })
                            style:width=format!("{}px", RECT_W)
                            style:height=format!("{}px", RECT_H)
                            on:mousedown=move |e| {
                                if let Some(stage) = stage_ref.get() {
                                    let rect = stage.get_bounding_client_rect();
                                    let mx = e.client_x() as f32 - rect.left() as f32;
                                    let my = e.client_y() as f32 - rect.top() as f32;
                                    rect_drag.set(Some(RectDragState {
                                        offset: (mx - RECT_HOME.0, my - RECT_HOME.1),
                                        current: RECT_HOME,
                                        moved: false,
                                        start: (mx, my),
                                    }));
                                    e.prevent_default();
                                }
                            }
                        >
                            <span class="ps-box-label">{label}</span>
                        </div>
                    }
                })}
            </div>
        </>
    }
}