mod rapier_world;
mod visual_confirmation;

use leptos::html::Div;
use leptos::prelude::*;
use rapier_world::RapierWorld;
use visual_confirmation::{Confirmation, FourLines, Rect};

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

const STAGE_W: f32 = 900.0;
const STAGE_H: f32 = 620.0;

const VACUUM_ID: usize = 100;
const VACUUM_W: f32 = 100.0;
const VACUUM_H: f32 = 320.0;
const VACUUM_Y: f32 = (STAGE_H - VACUUM_H) / 2.0;

const DRAG_THRESHOLD: f32 = 4.0;
const VACUUM_PROXIMITY: f32 = 40.0;
const DROP_BAND: f32 = 30.0;

const BOX_W: f32 = 170.0;
const BOX_H: f32 = 90.0;
const BOX_STEP: f32 = 120.0;

const RECT_W: f32 = 60.0;
const RECT_H: f32 = 80.0;
const RECT_HOME: (f32, f32) = (20.0, VACUUM_Y + (VACUUM_H - RECT_H) / 2.0);

#[derive(Clone, Copy)]
struct BoxData {
    id: usize,
    home: (f32, f32),
    size: (f32, f32),
    label: &'static str,
}

#[derive(Clone, Copy)]
struct Column {
    x: f32,
    y_start: f32,
}

const COLUMNS: [Column; 2] = [
    Column { x: 200.0, y_start: 70.0 },
    Column { x: 530.0, y_start: 110.0 },
];

fn column_of(x: f32) -> Option<usize> {
    COLUMNS.iter().position(|c| (x - c.x).abs() < 1.0)
}

fn initial_boxes() -> Vec<BoxData> {
    vec![
        BoxData { id: 0, home: (200.0, 70.0), size: (BOX_W, BOX_H), label: "Alpha" },
        BoxData { id: 1, home: (200.0, 190.0), size: (BOX_W, BOX_H), label: "Bravo" },
        BoxData { id: 2, home: (200.0, 310.0), size: (BOX_W, BOX_H), label: "Charlie" },
        BoxData { id: 3, home: (200.0, 430.0), size: (BOX_W, BOX_H), label: "Delta" },
        BoxData { id: 4, home: (530.0, 110.0), size: (BOX_W, BOX_H), label: "Echo" },
        BoxData { id: 5, home: (530.0, 230.0), size: (BOX_W, BOX_H), label: "Foxtrot" },
        BoxData { id: 6, home: (530.0, 350.0), size: (BOX_W, BOX_H), label: "Golf" },
        BoxData { id: 7, home: (530.0, 470.0), size: (BOX_W, BOX_H), label: "Hotel" },
    ]
}

#[derive(Clone, Copy)]
struct DragState {
    id: usize,
    offset: (f32, f32),
    current: (f32, f32),
    moved: bool,
}

#[derive(Clone, Copy)]
struct RectDragState {
    offset: (f32, f32),
    current: (f32, f32),
    moved: bool,
}

fn rect_viewport(b: &BoxData, ox: f32, oy: f32) -> Rect {
    Rect {
        x: ox + b.home.0,
        y: oy + b.home.1,
        w: b.size.0,
        h: b.size.1,
    }
}

fn rebuild_world(rapier: &RapierWorld, boxes: &[BoxData]) {
    let tuples: Vec<(usize, (f32, f32), (f32, f32))> =
        boxes.iter().map(|b| (b.id, b.home, b.size)).collect();
    rapier.rebuild(&tuples);
}

/// Remove `dragged` from wherever it was, insert it into `target_col` after
/// `above_id` (or at the top if `above_id` is None), then reassign Y positions
/// for every box in that column.
fn compact_column(boxes: &mut Vec<BoxData>, col_idx: usize) {
    let col = &COLUMNS[col_idx];
    let mut in_col: Vec<usize> = boxes
        .iter()
        .enumerate()
        .filter(|(_, b)| (b.home.0 - col.x).abs() < 1.0)
        .map(|(i, _)| i)
        .collect();
    in_col.sort_by(|&a, &b| boxes[a].home.1.partial_cmp(&boxes[b].home.1).unwrap());
    for (slot, i) in in_col.iter().enumerate() {
        boxes[*i].home = (col.x, col.y_start + slot as f32 * BOX_STEP);
    }
}

fn drop_into_column(
    boxes: &mut Vec<BoxData>,
    dragged: BoxData,
    target_col: usize,
    above_id: Option<usize>,
) {
    boxes.retain(|b| b.id != dragged.id);

    for c in 0..COLUMNS.len() {
        compact_column(boxes, c);
    }

    let col = &COLUMNS[target_col];

    let mut in_col: Vec<usize> = boxes
        .iter()
        .enumerate()
        .filter(|(_, b)| (b.home.0 - col.x).abs() < 1.0)
        .map(|(i, _)| i)
        .collect();
    in_col.sort_by(|&a, &b| boxes[a].home.1.partial_cmp(&boxes[b].home.1).unwrap());

    let insert_at = match above_id {
        Some(a) => in_col
            .iter()
            .position(|&i| boxes[i].id == a)
            .map(|p| p + 1)
            .unwrap_or(in_col.len()),
        None => 0,
    };

    let mut new_order: Vec<usize> = in_col.iter().map(|&i| boxes[i].id).collect();
    new_order.insert(insert_at, dragged.id);

    for (slot, id) in new_order.iter().enumerate() {
        let y = col.y_start + slot as f32 * BOX_STEP;
        if let Some(b) = boxes.iter_mut().find(|b| b.id == *id) {
            b.home = (col.x, y);
        } else {
            boxes.push(BoxData {
                id: dragged.id,
                home: (col.x, y),
                size: dragged.size,
                label: dragged.label,
            });
        }
    }
}

#[component]
fn App() -> impl IntoView {
    let boxes = RwSignal::new(initial_boxes());
    let drag: RwSignal<Option<DragState>> = RwSignal::new(None);
    let rect_drag: RwSignal<Option<RectDragState>> = RwSignal::new(None);
    let stored_in_vacuum: RwSignal<Option<BoxData>> = RwSignal::new(None);
    let confirmation: RwSignal<Option<Confirmation>> = RwSignal::new(None);
    let viz_on: RwSignal<bool> = RwSignal::new(true);

    let rapier = RapierWorld::new();
    rebuild_world(&rapier, &boxes.get_untracked());

    let stage_ref = NodeRef::<Div>::new();

    let on_mouse_move = move |e: web_sys::MouseEvent| {
        let Some(stage) = stage_ref.get() else { return; };
        let rect = stage.get_bounding_client_rect();
        let mx = e.client_x() as f32 - rect.left() as f32;
        let my = e.client_y() as f32 - rect.top() as f32;

        if let Some(mut d) = drag.get() {
            let new_current = (mx - d.offset.0, my - d.offset.1);
            if !d.moved
                && ((new_current.0 - d.current.0).abs() > DRAG_THRESHOLD
                    || (new_current.1 - d.current.1).abs() > DRAG_THRESHOLD)
            {
                d.moved = true;
            }
            d.current = new_current;
            drag.set(Some(d));
        }

        if let Some(mut r) = rect_drag.get() {
            let new_current = (mx - r.offset.0, my - r.offset.1);
            if !r.moved
                && ((new_current.0 - r.current.0).abs() > DRAG_THRESHOLD
                    || (new_current.1 - r.current.1).abs() > DRAG_THRESHOLD)
            {
                r.moved = true;
            }
            r.current = new_current;
            rect_drag.set(Some(r));
        }
    };

    let on_mouse_up = move |e: web_sys::MouseEvent| {
        let Some(stage) = stage_ref.get() else { return; };
        let r = stage.get_bounding_client_rect();
        let ox = r.left() as f32;
        let oy = r.top() as f32;
        let mx = e.client_x() as f32 - ox;
        let my = e.client_y() as f32 - oy;

        let over_vacuum = mx >= -VACUUM_PROXIMITY
            && mx <= VACUUM_W + VACUUM_PROXIMITY
            && my >= VACUUM_Y - VACUUM_PROXIMITY
            && my <= VACUUM_Y + VACUUM_H + VACUUM_PROXIMITY;

        // ---- box drop ----
        if let Some(d) = drag.get() {
            let snapshot = boxes.get();
            let Some(dragged) = snapshot.iter().find(|b| b.id == d.id).copied() else {
                drag.set(None);
                return;
            };

            if over_vacuum {
                let old: Vec<(usize, (f32, f32))> =
                    snapshot.iter().map(|b| (b.id, b.home)).collect();
                boxes.update(|bs| bs.retain(|b| b.id != d.id));
                rebuild_world(&rapier, &boxes.get_untracked());
                stored_in_vacuum.set(Some(dragged));

                let mut rects: Vec<Rect> = Vec::new();
                for b in boxes.get_untracked() {
                    let changed = old
                        .iter()
                        .find(|(id, _)| *id == b.id)
                        .map(|(_, p)| *p != b.home)
                        .unwrap_or(true);
                    if changed {
                        rects.push(rect_viewport(&b, ox, oy));
                    }
                }
                if viz_on.get() {
                    confirmation.set(Some(Confirmation::Many(rects)));
                }
            } else if !d.moved {
                if viz_on.get() {
                    confirmation.set(Some(Confirmation::One(rect_viewport(&dragged, ox, oy))));
                }
            } else {
                let (above, below) =
                    rapier.find_above_and_below((d.current, dragged.size), &[d.id], DROP_BAND);

                if above.is_some() || below.is_some() {
                    let anchor_id = above.or(below).unwrap();
                    let anchor = snapshot.iter().find(|b| b.id == anchor_id).unwrap();
                    let target_col = column_of(anchor.home.0).unwrap();

                    let old: Vec<(usize, (f32, f32))> =
                        snapshot.iter().map(|b| (b.id, b.home)).collect();

                    boxes.update(|bs| drop_into_column(bs, dragged, target_col, above));
                    rebuild_world(&rapier, &boxes.get_untracked());

                    let mut rects: Vec<Rect> = Vec::new();
                    for b in boxes.get_untracked() {
                        let changed = old
                            .iter()
                            .find(|(id, _)| *id == b.id)
                            .map(|(_, p)| *p != b.home)
                            .unwrap_or(true);
                        if changed {
                            rects.push(rect_viewport(&b, ox, oy));
                        }
                    }
                    if viz_on.get() {
                        confirmation.set(Some(Confirmation::Many(rects)));
                    }
                }
            }
            drag.set(None);
        }

        // ---- vacuum rect drop ----
        if let Some(rd) = rect_drag.get() {
            let Some(stored) = stored_in_vacuum.get() else {
                rect_drag.set(None);
                return;
            };
            if !rd.moved {
                rect_drag.set(None);
                return;
            }

            let (above, below) = rapier.find_above_and_below(
                (rd.current, (BOX_W, BOX_H)),
                &[VACUUM_ID],
                DROP_BAND,
            );

            if above.is_none() && below.is_none() {
                rect_drag.set(None);
            } else {
                let anchor_id = above.or(below).unwrap();
                let snapshot = boxes.get();
                let anchor = snapshot.iter().find(|b| b.id == anchor_id).unwrap();
                let target_col = column_of(anchor.home.0).unwrap();

                let old: Vec<(usize, (f32, f32))> =
                    snapshot.iter().map(|b| (b.id, b.home)).collect();

                boxes.update(|bs| drop_into_column(bs, stored, target_col, above));
                stored_in_vacuum.set(None);
                rebuild_world(&rapier, &boxes.get_untracked());

                let mut rects: Vec<Rect> = Vec::new();
                for b in boxes.get_untracked() {
                    let changed = old
                        .iter()
                        .find(|(id, _)| *id == b.id)
                        .map(|(_, p)| *p != b.home)
                        .unwrap_or(true);
                    if changed {
                        rects.push(rect_viewport(&b, ox, oy));
                    }
                }
                if viz_on.get() {
                    confirmation.set(Some(Confirmation::Many(rects)));
                }
                rect_drag.set(None);
            }
        }
    };

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
                style:width=format!("{}px", STAGE_W)
                style:height=format!("{}px", STAGE_H)
                on:mousemove=on_mouse_move
                on:mouseup=on_mouse_up
                on:mouseleave=move |_| {
                    drag.set(None);
                    rect_drag.set(None);
                }
            >
                <FourLines confirmation=confirmation />
                <For
                    each=move || boxes.get()
                    key=|b| b.id
                    children=move |b: BoxData| {
                        let id = b.id;
                        let home = b.home;
                        let size = b.size;
                        let label = b.label;
                        view! {
                            <div
                                class="box"
                                style:left=move || {
                                    let p = match drag.get() {
                                        Some(d) if d.id == id => d.current,
                                        _ => boxes
                                            .get()
                                            .iter()
                                            .find(|x| x.id == id)
                                            .map(|x| x.home)
                                            .unwrap_or(home),
                                    };
                                    format!("{}px", p.0)
                                }
                                style:top=move || {
                                    let p = match drag.get() {
                                        Some(d) if d.id == id => d.current,
                                        _ => boxes
                                            .get()
                                            .iter()
                                            .find(|x| x.id == id)
                                            .map(|x| x.home)
                                            .unwrap_or(home),
                                    };
                                    format!("{}px", p.1)
                                }
                                style:width=format!("{}px", size.0)
                                style:height=format!("{}px", size.1)
                                on:mousedown=move |e| {
                                    if let Some(stage) = stage_ref.get() {
                                        let rect = stage.get_bounding_client_rect();
                                        let mx = e.client_x() as f32 - rect.left() as f32;
                                        let my = e.client_y() as f32 - rect.top() as f32;
                                        let live = boxes
                                            .get()
                                            .iter()
                                            .find(|x| x.id == id)
                                            .map(|x| x.home)
                                            .unwrap_or(home);
                                        drag.set(Some(DragState {
                                            id,
                                            offset: (mx - live.0, my - live.1),
                                            current: live,
                                            moved: false,
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
                    style:top=format!("{}px", VACUUM_Y)
                    style:width=format!("{}px", VACUUM_W)
                    style:height=format!("{}px", VACUUM_H)
                />
                {move || stored_in_vacuum.get().map(|stored| {
                    let label = stored.label;
                    view! {
                        <div
                            class="vacuum-rect"
                            style:left=move || {
                                let p = match rect_drag.get() {
                                    Some(r) => r.current,
                                    None => RECT_HOME,
                                };
                                format!("{}px", p.0)
                            }
                            style:top=move || {
                                let p = match rect_drag.get() {
                                    Some(r) => r.current,
                                    None => RECT_HOME,
                                };
                                format!("{}px", p.1)
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