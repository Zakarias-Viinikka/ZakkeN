mod cascade;
mod layout;
mod rapier_world;
mod visual_confirmation;

use leptos::html::Div;
use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use cascade::release_pass;
use layout::{
    commit, stage_height, Block, Phase, BLOCK_W, BLOCK_X, PAGE_SLOP, PAGE_TOP, PAGE_W, PAGE_X,
    RECT_H, RECT_HOME, RECT_W, VACUUM_PROXIMITY, VACUUM_W,
};
use rapier_world::{RapierWorld, PAGE_ID_BASE};
use visual_confirmation::{Confirmation, FourLines, Rect};

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

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

fn rebuild_world(rapier: &RapierWorld, blocks: &[Block]) {
    let mut tuples: Vec<(usize, (f32, f32), (f32, f32))> = blocks
        .iter()
        .map(|b| (b.id, (b.x, b.target_y), (BLOCK_W, b.h)))
        .collect();
    for page in 0..2 {
        let x0 = PAGE_X[page] - PAGE_SLOP;
        let x1 = PAGE_X[page] + PAGE_W + PAGE_SLOP;
        tuples.push((
            PAGE_ID_BASE + page,
            (x0, PAGE_TOP),
            (x1 - x0, REGION_H),
        ));
    }
    rapier.rebuild(&tuples);
}

fn initial_blocks() -> Vec<Block> {
    let specs: [(usize, &'static str, f32, usize); 10] = [
        (0, "Alpha", 84.0, 0),
        (1, "Bravo", 120.0, 0),
        (2, "Charlie", 72.0, 0),
        (3, "Delta", 104.0, 0),
        (4, "Echo", 92.0, 0),
        (5, "Foxtrot", 112.0, 1),
        (6, "Golf", 76.0, 1),
        (7, "Hotel", 132.0, 1),
        (8, "India", 88.0, 1),
        (9, "Juliet", 100.0, 1),
    ];
    let mut blocks: Vec<Block> = specs
        .iter()
        .map(|&(id, label, h, page)| Block {
            id,
            label,
            x: BLOCK_X[page],
            y: 0.0,
            target_y: 0.0,
            vel: 0.0,
            h,
            phase: Phase::Idle,
            prev: None,
        })
        .collect();
    let orders = [vec![0usize, 1, 2, 3, 4], vec![5usize, 6, 7, 8, 9]];
    let all_ids: Vec<usize> = blocks.iter().map(|b| b.id).collect();
    commit(&mut blocks, &orders, &all_ids);
    blocks
}

#[component]
fn App() -> impl IntoView {
    let initial = initial_blocks();
    let boxes = RwSignal::new(initial.clone());
    let drag: RwSignal<Option<DragState>> = RwSignal::new(None);
    let rect_drag: RwSignal<Option<RectDragState>> = RwSignal::new(None);
    let stored: RwSignal<Option<Block>> = RwSignal::new(None);
    let confirmation: RwSignal<Option<Confirmation>> = RwSignal::new(None);
    let viz_on: RwSignal<bool> = RwSignal::new(true);

    let rapier = RapierWorld::new();
    rebuild_world(&rapier, &initial);

    let stage_ref = NodeRef::<Div>::new();

    {
        let boxes = boxes;
        let cb = Closure::<dyn FnMut()>::new(move || {
            let mut bs = boxes.get_untracked();
            if cascade::step(&mut bs) {
                boxes.set(bs);
            }
        });
        web_sys::window()
            .unwrap()
            .set_interval_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                16,
            )
            .unwrap();
        cb.forget();
    }

    let on_mouse_move = move |e: web_sys::MouseEvent| {
        let Some(stage) = stage_ref.get() else {
            return;
        };
        let rect = stage.get_bounding_client_rect();
        let mx = e.client_x() as f32 - rect.left() as f32;
        let my = e.client_y() as f32 - rect.top() as f32;

        if let Some(mut d) = drag.get() {
            if !d.moved
                && ((mx - d.start.0).abs() > DRAG_THRESHOLD
                    || (my - d.start.1).abs() > DRAG_THRESHOLD)
            {
                d.moved = true;
            }
            d.current = (mx - d.offset.0, my - d.offset.1);
            drag.set(Some(d));
        }

        if let Some(mut r) = rect_drag.get() {
            if !r.moved
                && ((mx - r.start.0).abs() > DRAG_THRESHOLD
                    || (my - r.start.1).abs() > DRAG_THRESHOLD)
            {
                r.moved = true;
            }
            r.current = (mx - r.offset.0, my - r.offset.1);
            rect_drag.set(Some(r));
        }
    };

    let on_mouse_up = move |e: web_sys::MouseEvent| {
        let Some(stage) = stage_ref.get() else {
            return;
        };
        let rect = stage.get_bounding_client_rect();
        let ox = rect.left() as f32;
        let oy = rect.top() as f32;
        let mx = e.client_x() as f32 - ox;
        let my = e.client_y() as f32 - oy;

        // --- block drag ---
        if let Some(d) = drag.get() {
            let snapshot = boxes.get_untracked();
            let Some(dragged) = snapshot.iter().find(|b| b.id == d.id).cloned() else {
                drag.set(None);
                return;
            };
            let in_vacuum = mx <= VACUUM_W + VACUUM_PROXIMITY;

            if !d.moved {
                if viz_on.get() {
                    confirmation.set(Some(Confirmation::One(Rect {
                        x: dragged.x + ox,
                        y: dragged.y + oy,
                        w: BLOCK_W,
                        h: dragged.h,
                    })));
                }
            } else if in_vacuum {
                let old_layout: Vec<(usize, f32, f32)> =
                    snapshot.iter().map(|b| (b.id, b.x, b.target_y)).collect();

                let mut new_blocks = snapshot.clone();
                new_blocks.retain(|b| b.id != d.id);

                let o0: Vec<usize> = rapier
                    .page_order(0)
                    .into_iter()
                    .filter(|&i| i != d.id)
                    .collect();
                let o1: Vec<usize> = rapier
                    .page_order(1)
                    .into_iter()
                    .filter(|&i| i != d.id)
                    .collect();

                commit(&mut new_blocks, &[o0, o1], &[]);
                release_pass(&mut new_blocks);
                stored.set(Some(dragged));
                boxes.set(new_blocks.clone());
                rebuild_world(&rapier, &new_blocks);

                if viz_on.get() {
                    let mut rects = Vec::new();
                    for b in &new_blocks {
                        if let Some(&(_, px, py)) = old_layout.iter().find(|(i, _, _)| *i == b.id)
                        {
                            if (b.x - px).abs() > 0.5 || (b.target_y - py).abs() > 0.5 {
                                rects.push(Rect {
                                    x: b.x + ox,
                                    y: b.target_y + oy,
                                    w: BLOCK_W,
                                    h: b.h,
                                });
                            }
                        }
                    }
                    if !rects.is_empty() {
                        confirmation.set(Some(Confirmation::Many(rects)));
                    }
                }
            } else if let Some((page, above)) =
                rapier.locate_drop((d.current.0 + BLOCK_W / 2.0, d.current.1 + dragged.h / 2.0), &[d.id])
            {
                let old_layout: Vec<(usize, f32, f32)> =
                    snapshot.iter().map(|b| (b.id, b.x, b.target_y)).collect();

                let mut order: Vec<usize> = rapier
                    .page_order(page)
                    .into_iter()
                    .filter(|&i| i != d.id)
                    .collect();
                let insert_at = match above {
                    None => 0,
                    Some(a) => order
                        .iter()
                        .position(|&i| i == a)
                        .map(|p| p + 1)
                        .unwrap_or(order.len()),
                };
                order.insert(insert_at, d.id);

                let other_page = 1 - page;
                let other_order: Vec<usize> = rapier
                    .page_order(other_page)
                    .into_iter()
                    .filter(|&i| i != d.id)
                    .collect();

                let mut new_blocks = snapshot.clone();
                let orders = if page == 0 {
                    [order, other_order]
                } else {
                    [other_order, order]
                };
                commit(&mut new_blocks, &orders, &[d.id]);
                release_pass(&mut new_blocks);
                boxes.set(new_blocks.clone());
                rebuild_world(&rapier, &new_blocks);

                if viz_on.get() {
                    let mut rects = Vec::new();
                    for b in &new_blocks {
                        let is_dragged = b.id == d.id;
                        let changed = match old_layout.iter().find(|(i, _, _)| *i == b.id) {
                            Some(&(_, px, py)) => {
                                (b.x - px).abs() > 0.5 || (b.target_y - py).abs() > 0.5
                            }
                            None => true,
                        };
                        if is_dragged || changed {
                            rects.push(Rect {
                                x: b.x + ox,
                                y: b.target_y + oy,
                                w: BLOCK_W,
                                h: b.h,
                            });
                        }
                    }
                    if !rects.is_empty() {
                        confirmation.set(Some(Confirmation::Many(rects)));
                    }
                }
            }
            drag.set(None);
        }

        // --- vacuum rect drag ---
        if let Some(rd) = rect_drag.get() {
            let Some(stored_block) = stored.get() else {
                rect_drag.set(None);
                return;
            };
            if !rd.moved {
                rect_drag.set(None);
                return;
            }

            if let Some((page, above)) = rapier.locate_drop((rd.current.0 + RECT_W / 2.0, rd.current.1 + RECT_H / 2.0), &[]) {
                let mut order: Vec<usize> = rapier
                    .page_order(page)
                    .into_iter()
                    .filter(|&i| i != stored_block.id)
                    .collect();
                let insert_at = match above {
                    None => 0,
                    Some(a) => order
                        .iter()
                        .position(|&i| i == a)
                        .map(|p| p + 1)
                        .unwrap_or(order.len()),
                };
                order.insert(insert_at, stored_block.id);

                let other_page = 1 - page;
                let other_order: Vec<usize> = rapier
                    .page_order(other_page)
                    .into_iter()
                    .filter(|&i| i != stored_block.id)
                    .collect();

                let mut new_blocks = boxes.get_untracked();
                let old_layout: Vec<(usize, f32, f32)> = new_blocks
                    .iter()
                    .map(|b| (b.id, b.x, b.target_y))
                    .collect();

                new_blocks.push(stored_block.clone());

                let orders = if page == 0 {
                    [order, other_order]
                } else {
                    [other_order, order]
                };
                commit(&mut new_blocks, &orders, &[stored_block.id]);
                release_pass(&mut new_blocks);
                boxes.set(new_blocks.clone());
                stored.set(None);
                rebuild_world(&rapier, &new_blocks);

                if viz_on.get() {
                    let mut rects = Vec::new();
                    for b in &new_blocks {
                        let is_restored = b.id == stored_block.id;
                        let changed = match old_layout.iter().find(|(i, _, _)| *i == b.id) {
                            Some(&(_, px, py)) => {
                                (b.x - px).abs() > 0.5 || (b.target_y - py).abs() > 0.5
                            }
                            None => true,
                        };
                        if is_restored || changed {
                            rects.push(Rect {
                                x: b.x + ox,
                                y: b.target_y + oy,
                                w: BLOCK_W,
                                h: b.h,
                            });
                        }
                    }
                    if !rects.is_empty() {
                        confirmation.set(Some(Confirmation::Many(rects)));
                    }
                }
            }
            rect_drag.set(None);
        }
    };

    {
        let mm = on_mouse_move;
        let cb = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(
            move |e: web_sys::MouseEvent| mm(e),
        );
        web_sys::window()
            .unwrap()
            .add_event_listener_with_callback("mousemove", cb.as_ref().unchecked_ref())
            .unwrap();
        cb.forget();
    }
    {
        let mu = on_mouse_up;
        let cb = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(
            move |e: web_sys::MouseEvent| mu(e),
        );
        web_sys::window()
            .unwrap()
            .add_event_listener_with_callback("mouseup", cb.as_ref().unchecked_ref())
            .unwrap();
        cb.forget();
    }

    view! {
        <div class="stage-wrap">
            <label class="viz-toggle">
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
                class="stage"
                style:width=format!("{}px", layout::STAGE_W)
                style:height=move || format!("{}px", stage_height(&boxes.get()))
                on:mouseleave=move |_| {
                    drag.set(None);
                    rect_drag.set(None);
                }
            >
                <FourLines confirmation=confirmation />
                <div
                    class="page-panel"
                    style:left=format!("{}px", PAGE_X[0])
                    style:top=format!("{}px", PAGE_TOP)
                    style:width=format!("{}px", PAGE_W)
                    style:height=move || {
                        let h = stage_height(&boxes.get()) - PAGE_TOP - 20.0;
                        format!("{}px", h)
                    }
                >
                    <span class="page-title">"PAGE 1"</span>
                </div>
                <div
                    class="page-panel"
                    style:left=format!("{}px", PAGE_X[1])
                    style:top=format!("{}px", PAGE_TOP)
                    style:width=format!("{}px", PAGE_W)
                    style:height=move || {
                        let h = stage_height(&boxes.get()) - PAGE_TOP - 20.0;
                        format!("{}px", h)
                    }
                >
                    <span class="page-title">"PAGE 2"</span>
                </div>
                <For
                    each=move || boxes.get()
                    key=|b| b.id
                    children=move |b: Block| {
                        let id = b.id;
                        let label = b.label;
                        let h = b.h;

                        let class_now = move || {
                            let mut c = String::from("box");
                            if let Some(d) = drag.get() {
                                if d.id == id {
                                    c.push_str(" dragging");
                                }
                            }
                            c
                        };

                        let left_now = move || {
                            if let Some(d) = drag.get() {
                                if d.id == id {
                                    return format!("{}px", d.current.0);
                                }
                            }
                            let x = boxes
                                .get()
                                .iter()
                                .find(|x| x.id == id)
                                .map(|x| x.x)
                                .unwrap_or(0.0);
                            format!("{}px", x)
                        };

                        let top_now = move || {
                            if let Some(d) = drag.get() {
                                if d.id == id {
                                    return format!("{}px", d.current.1);
                                }
                            }
                            let y = boxes
                                .get()
                                .iter()
                                .find(|x| x.id == id)
                                .map(|x| x.y)
                                .unwrap_or(0.0);
                            format!("{}px", y)
                        };

                        view! {
                            <div
                                class=class_now
                                style:left=left_now
                                style:top=top_now
                                style:width=format!("{}px", BLOCK_W)
                                style:height=format!("{}px", h)
                                on:mousedown=move |e| {
                                    if let Some(stage) = stage_ref.get() {
                                        let rect = stage.get_bounding_client_rect();
                                        let mx = e.client_x() as f32 - rect.left() as f32;
                                        let my = e.client_y() as f32 - rect.top() as f32;
                                        let live = boxes
                                            .get_untracked()
                                            .iter()
                                            .find(|x| x.id == id)
                                            .map(|x| (x.x, x.y))
                                            .unwrap_or((0.0, 0.0));
                                        drag.set(Some(DragState {
                                            id,
                                            offset: (mx - live.0, my - live.1),
                                            current: live,
                                            moved: false,
                                            start: (mx, my),
                                        }));
                                        e.prevent_default();
                                    }
                                }
                            >
                                <span class="box-label">{label}</span>
                            </div>
                        }
                    }
                />
                <div
                    class="vacuum"
                    style:left="0"
                    style:top="0"
                    style:width=format!("{}px", VACUUM_W)
                    style:height="100%"
                />
                {move || stored.get().map(|s| {
                    let label = s.label;
                    view! {
                        <div
                            class="vacuum-rect"
                            style:left=move || {
                                if let Some(r) = rect_drag.get() {
                                    format!("{}px", r.current.0)
                                } else {
                                    format!("{}px", RECT_HOME.0)
                                }
                            }
                            style:top=move || {
                                if let Some(r) = rect_drag.get() {
                                    format!("{}px", r.current.1)
                                } else {
                                    format!("{}px", RECT_HOME.1)
                                }
                            }
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
                            <span class="box-label">{label}</span>
                        </div>
                    }
                })}
            </div>
        </div>
    }
}