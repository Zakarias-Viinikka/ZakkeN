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

const DRAG_THRESHOLD: f32 = 4.0;
const VACUUM_PROXIMITY: f32 = 40.0;

#[derive(Clone, Copy)]
struct BoxData {
    id: usize,
    home: (f32, f32),
    size: (f32, f32),
}

#[derive(Clone, Copy)]
struct DragState {
    id: usize,
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

#[component]
fn App() -> impl IntoView {
    let w = 170.0;
    let h = 90.0;
    let mut initial: Vec<BoxData> = vec![
        // Left list.
        BoxData { id: 0, home: (200.0, 70.0), size: (w, h) },
        BoxData { id: 1, home: (200.0, 190.0), size: (w, h) },
        BoxData { id: 2, home: (200.0, 310.0), size: (w, h) },
        BoxData { id: 3, home: (200.0, 430.0), size: (w, h) },
        // Right list, offset so the two read as separate columns.
        BoxData { id: 4, home: (530.0, 110.0), size: (w, h) },
        BoxData { id: 5, home: (530.0, 230.0), size: (w, h) },
        BoxData { id: 6, home: (530.0, 350.0), size: (w, h) },
        BoxData { id: 7, home: (530.0, 470.0), size: (w, h) },
    ];
    initial.push(BoxData {
        id: VACUUM_ID,
        home: (0.0, (STAGE_H - VACUUM_H) / 2.0),
        size: (VACUUM_W, VACUUM_H),
    });

    let boxes = RwSignal::new(initial);
    let drag: RwSignal<Option<DragState>> = RwSignal::new(None);
    let confirmation: RwSignal<Option<Confirmation>> = RwSignal::new(None);

    let rapier = RapierWorld::new();
    for b in boxes.get_untracked() {
        rapier.add_fixed_box(b.id, b.home, b.size);
    }

    let stage_ref = NodeRef::<Div>::new();

    let on_mouse_move = move |e: web_sys::MouseEvent| {
        let Some(stage) = stage_ref.get() else { return; };
        let rect = stage.get_bounding_client_rect();
        let mx = e.client_x() as f32 - rect.left() as f32;
        let my = e.client_y() as f32 - rect.top() as f32;

        let Some(mut d) = drag.get() else { return; };
        let new_current = (mx - d.offset.0, my - d.offset.1);
        if !d.moved {
            let dx = (new_current.0 - d.current.0).abs();
            let dy = (new_current.1 - d.current.1).abs();
            if dx > DRAG_THRESHOLD || dy > DRAG_THRESHOLD {
                d.moved = true;
            }
        }
        d.current = new_current;
        drag.set(Some(d));
    };

    let on_mouse_up = move |e: web_sys::MouseEvent| {
        let Some(d) = drag.get() else { return; };
        let Some(stage) = stage_ref.get() else { return; };
        let r = stage.get_bounding_client_rect();
        let ox = r.left() as f32;
        let oy = r.top() as f32;
        let mx = e.client_x() as f32 - ox;
        let my = e.client_y() as f32 - oy;

        let snapshot = boxes.get();

        let vacuum = snapshot.iter().find(|b| b.id == VACUUM_ID).copied();
        let over_vacuum = vacuum.map_or(false, |v| {
            mx >= v.home.0 - VACUUM_PROXIMITY
                && mx <= v.home.0 + v.size.0 + VACUUM_PROXIMITY
                && my >= v.home.1 - VACUUM_PROXIMITY
                && my <= v.home.1 + v.size.1 + VACUUM_PROXIMITY
        });

        if over_vacuum {
            if let Some(v) = vacuum {
                confirmation.set(Some(Confirmation::One(rect_viewport(&v, ox, oy))));
            }
        } else if !d.moved {
            if let Some(b) = snapshot.iter().find(|b| b.id == d.id) {
                confirmation.set(Some(Confirmation::One(rect_viewport(b, ox, oy))));
            }
        } else {
            let size = snapshot
                .iter()
                .find(|b| b.id == d.id)
                .map(|b| b.size)
                .unwrap_or((0.0, 0.0));
            let (above, below) =
                rapier.find_above_and_below((d.current, size), &[d.id, VACUUM_ID]);
            let mut rects: Vec<Rect> = Vec::new();
            for id in [above, below].into_iter().flatten() {
                if let Some(b) = snapshot.iter().find(|b| b.id == id) {
                    rects.push(rect_viewport(b, ox, oy));
                }
            }
            confirmation.set(Some(Confirmation::Many(rects)));
        }
        drag.set(None);
    };

    view! {
        <div class="stage-wrap">
            <div
                node_ref=stage_ref
                class="stage"
                style:width=format!("{}px", STAGE_W)
                style:height=format!("{}px", STAGE_H)
                on:mousemove=on_mouse_move
                on:mouseup=on_mouse_up
                on:mouseleave=move |_| drag.set(None)
            >
                <FourLines confirmation=confirmation />
                <For
                    each=move || boxes.get()
                    key=|b| b.id
                    children=move |b: BoxData| {
                        let id = b.id;
                        let home = b.home;
                        let size = b.size;
                        let is_vacuum = id == VACUUM_ID;
                        let class = if is_vacuum { "vacuum" } else { "box" };
                        view! {
                            <div
                                class=class
                                style:left=move || {
                                    let p = match drag.get() {
                                        Some(d) if d.id == id => d.current,
                                        _ => home,
                                    };
                                    format!("{}px", p.0)
                                }
                                style:top=move || {
                                    let p = match drag.get() {
                                        Some(d) if d.id == id => d.current,
                                        _ => home,
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
                                        drag.set(Some(DragState {
                                            id,
                                            offset: (mx - home.0, my - home.1),
                                            current: home,
                                            moved: false,
                                        }));
                                        e.prevent_default();
                                    }
                                }
                            />
                        }
                    }
                />
            </div>
        </div>
    }
}