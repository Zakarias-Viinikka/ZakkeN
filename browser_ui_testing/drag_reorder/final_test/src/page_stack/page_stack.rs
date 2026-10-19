use leptos::prelude::*;
use leptos::ev;
use leptos::html::Div;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use super::types::{PageInput, BlockInput, MoveEvent};
use super::layout::{Block, Phase, commit, stage_height, stage_width, page_x, page_panel_height, PAGE_W, PAGE_TOP, VACUUM_W, VACUUM_PROXIMITY, RECT_W, RECT_H, RECT_HOME, BLOCK_W, UNMEASURED_HEIGHT};
use super::cascade::{step, release_pass};
use super::rapier_world::{RapierWorld, PAGE_ID_BASE};
use super::visual_confirmation::{Confirmation, FourLines, Rect};

const DRAG_THRESHOLD: f32 = 4.0;
const REGION_H: f32 = 20000.0;

#[derive(Clone, Copy)]
struct DragState {
    id: usize,
    offset: (f32, f32),
    current: (f32, f32),
    moved: bool,
    start: (f32, f32),
}

#[derive(Clone, Copy)]
struct RectDragState {
    offset: (f32, f32),
    current: (f32, f32),
    moved: bool,
    start: (f32, f32),
}

fn rebuild_world(rapier: &RapierWorld, blocks: &[Block], page_count: usize) {
    let mut tuples: Vec<(usize, (f32, f32), (f32, f32))> = blocks
        .iter()
        .map(|b| {
            let h = if b.measured { b.h } else { UNMEASURED_HEIGHT };
            (b.internal_id, (b.x, b.target_y), (BLOCK_W, h))
        })
        .collect();
    for page in 0..page_count {
        let x0 = page_x(page) - 16.0;
        let x1 = page_x(page) + PAGE_W + 16.0;
        tuples.push((
            PAGE_ID_BASE + page,
            (x0, PAGE_TOP),
            (x1 - x0, REGION_H),
        ));
    }
    rapier.rebuild(&tuples);
}

fn apply_heights(
    pairs: Vec<(usize, f32)>,
    boxes: RwSignal<Vec<Block>>,
    rapier: &RapierWorld,
    page_count: RwSignal<usize>,
) {
    let mut bs = boxes.get_untracked();
    let mut any_moved = false;
    for (id, h) in pairs {
        if let Some(b) = bs.iter_mut().find(|b| b.internal_id == id) {
            if !b.measured || (b.h - h).abs() >= 0.5 {
                any_moved = true;
            }
            b.h = h;
            b.measured = true;
        }
    }
    if !any_moved {
        return;
    }
    let pc = page_count.get_untracked();
    let mut all_orders: Vec<Vec<usize>> = Vec::with_capacity(pc);
    for p in 0..pc {
        all_orders.push(rapier.page_order(p));
    }
    let snap_ids: Vec<usize> = bs.iter().map(|b| b.internal_id).collect();
    commit(&mut bs, &all_orders, &snap_ids);
    boxes.set(bs.clone());
    rebuild_world(rapier, &bs, pc);
}

#[allow(clippy::too_many_arguments)]
fn apply_move(
    moved_id: usize,
    dest_page: usize,
    above: Option<usize>,
    is_restore: bool,
    boxes: RwSignal<Vec<Block>>,
    rapier: &RapierWorld,
    stored: RwSignal<Option<usize>>,
    confirmation: RwSignal<Option<Confirmation>>,
    viz_on: RwSignal<bool>,
    page_count: RwSignal<usize>,
    current_page_ids: RwSignal<Vec<String>>,
    stage_ref: NodeRef<Div>,
    on_move: Callback<MoveEvent>,
) {
    let pc = page_count.get_untracked();
    let Some(src) = (0..pc).find(|&p| rapier.page_order(p).contains(&moved_id)) else {
        return;
    };

    let mut all_orders: Vec<Vec<usize>> = Vec::with_capacity(pc);
    for p in 0..pc {
        all_orders.push(
            rapier
                .page_order(p)
                .into_iter()
                .filter(|&i| i != moved_id)
                .collect(),
        );
    }
    let insert_at = match above {
        None => 0,
        Some(a) => all_orders[dest_page]
            .iter()
            .position(|&i| i == a)
            .map(|p| p + 1)
            .unwrap_or(all_orders[dest_page].len()),
    };
    all_orders[dest_page].insert(insert_at, moved_id);

    let mut bs = boxes.get_untracked();

    if is_restore {
        if let Some(b) = bs.iter_mut().find(|b| b.internal_id == moved_id) {
            b.ghost = false;
        }
        stored.set(None);
        leptos::logging::log!("vacuumed");
    }

    commit(&mut bs, &all_orders, &[moved_id]);
    release_pass(&mut bs);
    boxes.set(bs.clone());
    rebuild_world(rapier, &bs, pc);

    let dest_order = &all_orders[dest_page];
    let pos = dest_order.iter().position(|&i| i == moved_id).unwrap_or(0);
    let above_id = if pos > 0 { Some(dest_order[pos - 1]) } else { None };
    let below_id = if pos + 1 < dest_order.len() {
        Some(dest_order[pos + 1])
    } else {
        None
    };

    let moved_str = bs
        .iter()
        .find(|b| b.internal_id == moved_id)
        .map(|b| b.block_id.to_string())
        .unwrap_or_default();
    let pids = current_page_ids.get_untracked();
    let from_str = pids.get(src).cloned().unwrap_or_default();
    let to_str = pids.get(dest_page).cloned().unwrap_or_default();
    let above_str = above_id.and_then(|id| {
        bs.iter()
            .find(|b| b.internal_id == id)
            .map(|b| b.block_id.to_string())
    });
    let below_str = below_id.and_then(|id| {
        bs.iter()
            .find(|b| b.internal_id == id)
            .map(|b| b.block_id.to_string())
    });

    if viz_on.get_untracked() {
        if let Some(stage) = stage_ref.get() {
            let rect = stage.get_bounding_client_rect();
            let ox = rect.left() as f32;
            let oy = rect.top() as f32;
            let mut rects = Vec::new();
            for candidate in [above_id, below_id].into_iter().flatten() {
                if let Some(b) = bs.iter().find(|b| b.internal_id == candidate) {
                    rects.push(Rect {
                        x: b.x + ox,
                        y: b.target_y + oy,
                        w: BLOCK_W,
                        h: b.h,
                    });
                }
            }
            if rects.is_empty() {
                confirmation.set(None);
            } else {
                confirmation.set(Some(Confirmation::Many(rects)));
            }
        }
    }

    on_move.run(MoveEvent {
        moved_block_id: moved_str,
        from_page_id: from_str,
        to_page_id: to_str,
        above: above_str,
        below: below_str,
    });
}

#[component]
pub fn PageStack(
    #[prop(into)] pages: Signal<Vec<PageInput>>,
    #[prop(into)] blocks: Signal<Vec<BlockInput>>,
    on_move: Callback<MoveEvent>,
) -> impl IntoView {
    let boxes = RwSignal::new(Vec::<Block>::new());
    let drag: RwSignal<Option<DragState>> = RwSignal::new(None);
    let rect_drag: RwSignal<Option<RectDragState>> = RwSignal::new(None);
    let stored: RwSignal<Option<usize>> = RwSignal::new(None);
    let confirmation: RwSignal<Option<Confirmation>> = RwSignal::new(None);
    let viz_on: RwSignal<bool> = RwSignal::new(true);
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
        let observer =
            web_sys::ResizeObserver::new(cb.as_ref().unchecked_ref()).expect("ResizeObserver::new");
        cb.forget();
        observer_stored.set_value(Some(observer));
    }

    let rapier_effect = rapier.clone();
    Effect::new(move || {
        let incoming_pages = pages.get();
        let incoming_blocks = blocks.get();
        
        let current_boxes = boxes.get_untracked();
        let current_map = id_map.get_untracked();
        let current_pids = current_page_ids.get_untracked();
        
        let incoming_pids = incoming_pages.iter().map(|p| p.page_id.clone()).collect::<Vec<_>>();
        let mut incoming_arrangement = Vec::new();
        for page in &incoming_pages {
            let page_blocks = incoming_blocks.iter()
                .filter(|b| b.page_id == page.page_id)
                .map(|b| b.block_id.clone())
                .collect::<Vec<_>>();
            incoming_arrangement.push(page_blocks);
        }
        
        let mut current_arrangement = Vec::new();
        for p in 0..page_count.get_untracked() {
            current_arrangement.push(rapier_effect.page_order(p).into_iter().map(|id| {
                current_boxes.iter().find(|b| b.internal_id == id).map(|b| b.block_id.to_string()).unwrap_or_default()
            }).collect::<Vec<_>>());
        }
        
        if incoming_pids == current_pids && incoming_arrangement == current_arrangement {
            return;
        }
        
        let mut new_map = HashMap::new();
        let mut new_boxes = Vec::new();
        
        for page in &incoming_pages {
            for block in &incoming_blocks {
                if block.page_id != page.page_id { continue; }
                let internal_id = if let Some(&id) = current_map.get(&block.block_id) {
                    id
                } else {
                    let id = next_id.get_untracked();
                    next_id.set(id + 1);
                    id
                };
                new_map.insert(block.block_id.clone(), internal_id);
                
                if let Some(existing) = current_boxes.iter().find(|b| b.internal_id == internal_id) {
                    new_boxes.push(existing.clone());
                } else {
                    new_boxes.push(Block {
                        internal_id,
                        block_id: Arc::from(block.block_id.as_str()),
                        content: ArcRwSignal::new(block.content.clone()),
                        x: 0.0,
                        y: 0.0,
                        target_y: 0.0,
                        vel: 0.0,
                        h: UNMEASURED_HEIGHT,
                        phase: Phase::Idle,
                        prev: None,
                        ghost: false,
                        measured: false,
                    });
                }
            }
        }
        
        if let Some(stored_id) = stored.get_untracked() {
            if !new_map.values().any(|&id| id == stored_id) {
                stored.set(None);
            }
        }
        
        let orders: Vec<Vec<usize>> = incoming_pages.iter().map(|p| {
            incoming_blocks.iter()
                .filter(|b| b.page_id == p.page_id)
                .map(|b| new_map[&b.block_id])
                .collect()
        }).collect();
        
        let snap_ids = new_boxes.iter().map(|b| b.internal_id).collect::<Vec<_>>();
        commit(&mut new_boxes, &orders, &snap_ids);
        boxes.set(new_boxes.clone());
        current_page_ids.set(incoming_pids);
        page_count.set(incoming_pages.len());
        rebuild_world(&rapier_effect, &new_boxes, incoming_pages.len());
        
        drag.set(None);
        rect_drag.set(None);
        confirmation.set(None);
    });

    let interval_handle = set_interval_with_handle(move || {
        let mut bs = boxes.get_untracked();
        if step(&mut bs) {
            boxes.set(bs);
        }
    }, Duration::from_millis(16)).expect("failed to set interval");

    let on_mouse_move = move |e: ev::MouseEvent| {
        let Some(stage) = stage_ref.get() else { return; };
        let rect = stage.get_bounding_client_rect();
        let mx = e.client_x() as f32 - rect.left() as f32;
        let my = e.client_y() as f32 - rect.top() as f32;

        if let Some(mut d) = drag.get() {
            if !d.moved && ((mx - d.start.0).abs() > DRAG_THRESHOLD || (my - d.start.1).abs() > DRAG_THRESHOLD) {
                d.moved = true;
            }
            d.current = (mx - d.offset.0, my - d.offset.1);
            drag.set(Some(d));
        }

        if let Some(mut r) = rect_drag.get() {
            if !r.moved && ((mx - r.start.0).abs() > DRAG_THRESHOLD || (my - r.start.1).abs() > DRAG_THRESHOLD) {
                r.moved = true;
            }
            r.current = (mx - r.offset.0, my - r.offset.1);
            rect_drag.set(Some(r));
        }
    };

    let on_mouse_up = move |e: ev::MouseEvent| {
        let Some(stage) = stage_ref.get() else {
            drag.set(None);
            rect_drag.set(None);
            return;
        };
        let rect = stage.get_bounding_client_rect();
        let ox = rect.left() as f32;
        let oy = rect.top() as f32;
        let mx = e.client_x() as f32 - ox;
        let my = e.client_y() as f32 - oy;

        if let Some(d) = drag.get_untracked() {
            let snapshot = boxes.get_untracked();
            let dragged = snapshot.iter().find(|b| b.internal_id == d.id).cloned();
            if let Some(dragged) = dragged {
                let in_vacuum = mx <= VACUUM_W + VACUUM_PROXIMITY;

                if !d.moved {
                    if viz_on.get_untracked() {
                        confirmation.set(Some(Confirmation::One(Rect {
                            x: dragged.x + ox,
                            y: dragged.y + oy,
                            w: BLOCK_W,
                            h: dragged.h,
                        })));
                    }
                } else if in_vacuum {
                    let stored_id = stored.get_untracked();
                    if stored_id != Some(d.id) {
                        let mut new_boxes = boxes.get_untracked();
                        let mut rects: Vec<Rect> = Vec::new();
                        if let Some(prev) = stored_id {
                            if let Some(pb) = new_boxes.iter_mut().find(|b| b.internal_id == prev) {
                                pb.ghost = false;
                                rects.push(Rect {
                                    x: pb.x + ox,
                                    y: pb.target_y + oy,
                                    w: BLOCK_W,
                                    h: pb.h,
                                });
                            }
                        }
                        if let Some(db) = new_boxes.iter_mut().find(|b| b.internal_id == d.id) {
                            db.ghost = true;
                            rects.push(Rect {
                                x: db.x + ox,
                                y: db.target_y + oy,
                                w: BLOCK_W,
                                h: db.h,
                            });
                        }
                        boxes.set(new_boxes);
                        stored.set(Some(d.id));
                        if viz_on.get_untracked() && !rects.is_empty() {
                            confirmation.set(Some(Confirmation::Many(rects)));
                        }
                    }
                } else {
                    if let Some((page, above)) = rapier.locate_drop(
                        (d.current.0 + BLOCK_W / 2.0, d.current.1 + dragged.h / 2.0),
                        &[d.id],
                    ) {
                        apply_move(
                            d.id,
                            page,
                            above,
                            false,
                            boxes,
                            &rapier,
                            stored,
                            confirmation,
                            viz_on,
                            page_count,
                            current_page_ids,
                            stage_ref,
                            on_move,
                        );
                    }
                }
            }
            drag.set(None);
        }

        if let Some(rd) = rect_drag.get_untracked() {
            if rd.moved {
                if let Some(stored_id) = stored.get_untracked() {
                    if let Some((page, above)) = rapier.locate_drop(
                        (rd.current.0 + RECT_W / 2.0, rd.current.1 + RECT_H / 2.0),
                        &[stored_id],
                    ) {
                        apply_move(
                            stored_id,
                            page,
                            above,
                            true,
                            boxes,
                            &rapier,
                            stored,
                            confirmation,
                            viz_on,
                            page_count,
                            current_page_ids,
                            stage_ref,
                            on_move,
                        );
                    }
                }
            }
            rect_drag.set(None);
        }
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
        <div class="ps-stage-wrap">
            <style>{include_str!("page_stack.css")}</style>
            <label class="ps-viz-toggle">
                <input
                    type="checkbox"
                    prop:checked=move || viz_on.get()
                    on:change=move |e| {
                        let checked = event_target_checked(&e);
                        viz_on.set(checked);
                        if !checked {
                            confirmation.set(None);
                        }
                    }
                />
                "visual confirmation"
            </label>
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
                <FourLines confirmation=confirmation />
                <For
                    each=move || current_page_ids.get()
                    key=|pid| pid.clone()
                    children=move |pid| {
                        let page_idx = current_page_ids.get_untracked().iter().position(|p| p == &pid).unwrap_or(0);
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
                    children=move |b: Block| {
                        let id = b.internal_id;
                        let content = b.content.clone();
                        let initial_content = content.get_untracked();
                        let node_ref = NodeRef::<Div>::new();
                        let observer_for_mount = observer_stored;
                        
                        Effect::new(move |_| {
                            let Some(el) = node_ref.get() else { return; };
                            if let Ok(Some(ta)) = el.query_selector("textarea") {
                                let ta: web_sys::HtmlTextAreaElement = ta.unchecked_into();
                                let _ = web_sys::HtmlElement::style(&ta).set_property("height", "auto");
                                let sh = ta.scroll_height();
                                let _ = web_sys::HtmlElement::style(&ta).set_property("height", &format!("{}px", sh));
                            }
                            observer_for_mount.with_value(|opt| {
                                if let Some(obs) = opt {
                                    let _ = obs.observe(&el);
                                }
                            });
                        });
                        
                        let class_now = move || {
                            let mut c = String::from("ps-box");
                            if let Some(cur) = boxes.get().iter().find(|x| x.internal_id == id) {
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
                        
                        let disabled_now = move || {
                            boxes.get().iter().find(|x| x.internal_id == id).map(|x| x.ghost).unwrap_or(false)
                        };
                        
                        let on_input = move |e: ev::Event| {
                            let Some(t) = e.target() else { return; };
                            let ta: web_sys::HtmlTextAreaElement = t.unchecked_into();
                            content.set(ta.value());
                            let _ = web_sys::HtmlElement::style(&ta).set_property("height", "auto");
                            let sh = ta.scroll_height();
                            let _ = web_sys::HtmlElement::style(&ta).set_property("height", &format!("{}px", sh));
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
                                <div class="ps-handle" on:mousedown=move |e| {
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
                                <textarea
                                    class="ps-content"
                                    prop:value=initial_content
                                    prop:disabled=disabled_now
                                    on:input=on_input
                                ></textarea>
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
        </div>
    }
}