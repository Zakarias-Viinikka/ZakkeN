use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

const STAGE_W: f32 = 900.0;
const STAGE_H: f32 = 900.0;

const BOX_W: f32 = 220.0;
const BOX_H: f32 = 100.0;
const BOX_STEP: f32 = 130.0;
const BOX_X: f32 = (STAGE_W - BOX_W) / 2.0;
const BOX_Y0: f32 = 60.0;
const OFFSET: f32 = BOX_STEP;

const EDGE_PAD: f32 = 10.0;
const MIN_Y: f32 = EDGE_PAD;
const MAX_Y: f32 = STAGE_H - BOX_H - EDGE_PAD;

const ACCEL: f32 = 1500.0;
const MAX_V: f32 = 600.0;
const PROXIMITY: f32 = 2.0;
const DT: f32 = 0.016;

/// Boundary refusers are effectively unbeatable. Chained refusers decrease
/// from here outward.
const BOUNDARY_SENIORITY: u32 = 1000;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    Idle,
    Moving,
    /// Box refuses to move in `dir` (the direction it would be pushed).
    /// `seniority` is the strength of the refusal — a mover with strictly
    /// higher seniority can override it; otherwise the mover is stopped and
    /// snapped back one step.
    Refusing { dir: i8, seniority: u32 },
}

#[derive(Clone)]
struct BoxState {
    id: usize,
    x: f32,
    y: f32,
    label: &'static str,
    mode: Mode,
    dir: i8,
    target_y: f32,
    vel: f32,
    seniority: u32,
    queued: bool,
}

fn initial_boxes() -> Vec<BoxState> {
    const LABELS: [&str; 5] = ["One", "Two", "Three", "Four", "Five"];
    LABELS
        .iter()
        .enumerate()
        .map(|(i, label)| {
            let y = BOX_Y0 + i as f32 * BOX_STEP;
            BoxState {
                id: i,
                x: BOX_X,
                y,
                label,
                mode: Mode::Idle,
                dir: 0,
                target_y: y,
                vel: 0.0,
                seniority: 0,
                queued: false,
            }
        })
        .collect()
}

fn compute_target(y: f32, dir: i8) -> f32 {
    y + dir as f32 * OFFSET
}

fn propagate(bs: &mut [BoxState]) {
    let triggers: Vec<(usize, f32, i8, u32)> = bs
        .iter()
        .filter(|b| !b.queued && b.mode == Mode::Moving)
        .map(|b| (b.id, b.vel, b.dir, b.seniority))
        .collect();

    for (b_id, b_vel, b_dir, b_sen) in triggers {
        let Some(b) = bs.iter().find(|x| x.id == b_id) else {
            continue;
        };
        let b_cy = b.y + BOX_H / 2.0;
        let b_y = b.y;

        let mut best: Option<(f32, usize)> = None;
        for other in bs.iter() {
            if other.id == b_id || other.queued {
                continue;
            }
            let o_cy = other.y + BOX_H / 2.0;
            let diff = (o_cy - b_cy) * b_dir as f32;
            if diff <= 0.0 {
                continue;
            }
            if best.map_or(true, |(d, _)| diff < d) {
                best = Some((diff, other.id));
            }
        }

        let Some((_, ahead_id)) = best else {
            continue;
        };
        let Some(ahead) = bs.iter().find(|x| x.id == ahead_id) else {
            continue;
        };
        if ahead.mode != Mode::Idle {
            continue;
        }
        let ahead_y = ahead.y;

        let gap = if b_dir > 0 {
            ahead_y - (b_y + BOX_H)
        } else {
            b_y - (ahead_y + BOX_H)
        };
        if gap > PROXIMITY {
            continue;
        }

        let b_target = b.target_y;
        if let Some(a) = bs.iter_mut().find(|x| x.id == ahead_id) {
            a.mode = Mode::Moving;
            a.dir = b_dir;
            a.target_y = b_target + b_dir as f32 * BOX_STEP;
            a.vel = b_vel;
            a.seniority = b_sen.saturating_add(1);
        }
    }
}

/// Each refusing box pushes the single closest touching non-refusing box on
/// its pressure side back one step relative to itself, and turns that box
/// into a refuser with seniority + 1. Loops until stable so cascades spread
/// in a single frame.
fn spread_refusal(bs: &mut [BoxState]) {
    loop {
        let mut changed = false;
        let refusers: Vec<(usize, i8, u32, f32)> = bs
            .iter()
            .filter_map(|b| match b.mode {
                Mode::Refusing { dir, seniority } => Some((b.id, dir, seniority, b.y)),
                _ => None,
            })
            .collect();

        for (r_id, r_dir, r_sen, r_y) in refusers {
            let r_cy = r_y + BOX_H / 2.0;

            let mut best: Option<(f32, usize)> = None;
            for b in bs.iter() {
                if b.id == r_id || b.queued {
                    continue;
                }
                if matches!(b.mode, Mode::Refusing { .. }) {
                    continue;
                }
                let b_cy = b.y + BOX_H / 2.0;
                let diff = (b_cy - r_cy) * -(r_dir as f32);
                if diff <= 0.0 {
                    continue;
                }
                let gap = if r_dir > 0 {
                    r_y - (b.y + BOX_H)
                } else {
                    b.y - (r_y + BOX_H)
                };
                if gap > PROXIMITY {
                    continue;
                }
                if best.map_or(true, |(d, _)| diff < d) {
                    best = Some((diff, b.id));
                }
            }

            let Some((_, pushed_id)) = best else {
                continue;
            };
            let target_y = r_y - r_dir as f32 * BOX_STEP;
            if let Some(b) = bs.iter_mut().find(|x| x.id == pushed_id) {
                b.y = target_y;
                b.vel = 0.0;
                b.dir = 0;
                b.mode = Mode::Refusing {
                    dir: r_dir,
                    seniority: r_sen.saturating_add(1),
                };
                changed = true;
            }
        }

        if !changed {
            break;
        }
    }
}

/// A moving box that reaches a Refusing box either overrides it (direction
/// is opposite to its refuse direction, or seniority is higher) or is stopped
/// one step behind it and turns into a refuser itself.
fn resolve_mover_refuser(bs: &mut [BoxState]) {
    let movers: Vec<(usize, i8, f32, u32, f32)> = bs
        .iter()
        .filter(|b| !b.queued && b.mode == Mode::Moving)
        .map(|b| (b.id, b.dir, b.vel, b.seniority, b.y))
        .collect();

    for (m_id, m_dir, m_vel, m_sen, m_y) in movers {
        let m_cy = m_y + BOX_H / 2.0;

        let mut best: Option<(f32, usize)> = None;
        for other in bs.iter() {
            if other.id == m_id || other.queued {
                continue;
            }
            let o_cy = other.y + BOX_H / 2.0;
            let d = (o_cy - m_cy) * m_dir as f32;
            if d <= 0.0 {
                continue;
            }
            if best.map_or(true, |(bd, _)| d < bd) {
                best = Some((d, other.id));
            }
        }

        let Some((_, ahead_id)) = best else {
            continue;
        };
        let Some(ahead) = bs.iter().find(|x| x.id == ahead_id) else {
            continue;
        };
        let ahead_y = ahead.y;
        let ahead_mode = ahead.mode;

        let gap = if m_dir > 0 {
            ahead_y - (m_y + BOX_H)
        } else {
            m_y - (ahead_y + BOX_H)
        };
        if gap > PROXIMITY {
            continue;
        }

        if let Mode::Refusing {
            dir: r_dir,
            seniority: r_sen,
        } = ahead_mode
        {
            let direction_ok = m_dir != r_dir;
            let seniority_ok = m_sen > r_sen;

            if direction_ok || seniority_ok {
                if let Some(a) = bs.iter_mut().find(|x| x.id == ahead_id) {
                    a.mode = Mode::Moving;
                    a.dir = m_dir;
                    a.target_y = compute_target(a.y, m_dir);
                    a.vel = m_vel;
                    a.seniority = m_sen.saturating_add(1);
                }
            } else {
                let m_target_y = ahead_y - m_dir as f32 * BOX_STEP;
                if let Some(m) = bs.iter_mut().find(|x| x.id == m_id) {
                    m.y = m_target_y;
                    m.mode = Mode::Refusing {
                        dir: r_dir,
                        seniority: r_sen.saturating_add(1),
                    };
                    m.vel = 0.0;
                    m.seniority = r_sen.saturating_add(1);
                }
            }
        }
    }
}

fn tick(boxes: &RwSignal<Vec<BoxState>>, click_queue: &RwSignal<Option<(usize, i8)>>) {
    let mut bs = boxes.get_untracked();
    if bs.is_empty() {
        return;
    }

    // 1. Advance movers; convert boundary hits to refusing.
    for b in bs.iter_mut() {
        if b.queued || b.mode != Mode::Moving {
            continue;
        }
        b.vel = (b.vel + ACCEL * DT).min(MAX_V);
        b.y += b.dir as f32 * b.vel * DT;

        if b.dir < 0 && b.y <= MIN_Y {
            b.y = MIN_Y;
            b.mode = Mode::Refusing {
                dir: -1,
                seniority: BOUNDARY_SENIORITY,
            };
            b.vel = 0.0;
            continue;
        }
        if b.dir > 0 && b.y >= MAX_Y {
            b.y = MAX_Y;
            b.mode = Mode::Refusing {
                dir: 1,
                seniority: BOUNDARY_SENIORITY,
            };
            b.vel = 0.0;
            continue;
        }

        let reached = (b.dir < 0 && b.y <= b.target_y) || (b.dir > 0 && b.y >= b.target_y);
        if reached {
            b.y = b.target_y;
            b.mode = Mode::Idle;
            b.vel = 0.0;
        }
    }

    propagate(&mut bs);
    spread_refusal(&mut bs);
    resolve_mover_refuser(&mut bs);
    spread_refusal(&mut bs);

    let settled = !bs.iter().any(|b| !b.queued && b.mode == Mode::Moving);
    boxes.set(bs);

    if settled {
        if let Some((id, dir)) = click_queue.get_untracked() {
            click_queue.set(None);
            start_move(boxes, id, dir);
        }
    }
}

fn start_move(boxes: &RwSignal<Vec<BoxState>>, id: usize, dir: i8) {
    boxes.update(|bs| {
        if let Some(b) = bs.iter_mut().find(|b| b.id == id) {
            if let Mode::Refusing { dir: r_dir, .. } = b.mode {
                if dir == r_dir {
                    return;
                }
            }
            b.mode = Mode::Moving;
            b.dir = dir;
            b.target_y = compute_target(b.y, dir);
            b.vel = 0.0;
            b.seniority = 0;
            b.queued = false;
        }
    });
}

fn handle_click(
    boxes: &RwSignal<Vec<BoxState>>,
    click_queue: &RwSignal<Option<(usize, i8)>>,
    id: usize,
    dir: i8,
) {
    let snapshot = boxes.get_untracked();
    let any_active = snapshot
        .iter()
        .any(|b| !b.queued && b.mode == Mode::Moving);

    if any_active {
        click_queue.set(Some((id, dir)));
        boxes.update(|bs| {
            if let Some(b) = bs.iter_mut().find(|b| b.id == id) {
                if b.mode == Mode::Idle {
                    b.queued = true;
                }
            }
        });
    } else {
        start_move(boxes, id, dir);
    }
}

#[component]
fn App() -> impl IntoView {
    let boxes = RwSignal::new(initial_boxes());
    let click_queue: RwSignal<Option<(usize, i8)>> = RwSignal::new(None);

    let cb = Closure::<dyn FnMut()>::new(move || {
        tick(&boxes, &click_queue);
    });
    web_sys::window()
        .unwrap()
        .set_interval_with_callback_and_timeout_and_arguments_0(
            cb.as_ref().unchecked_ref(),
            16,
        )
        .unwrap();
    cb.forget();

    view! {
        <div class="stage-wrap">
            <div
                class="stage"
                style:width=format!("{}px", STAGE_W)
                style:height=format!("{}px", STAGE_H)
            >
                <For
                    each=move || boxes.get()
                    key=|b| b.id
                    children=move |b: BoxState| {
                        let id = b.id;
                        let label = b.label;

                        let class_now = move || {
                            let live = boxes.get();
                            let Some(b) = live.iter().find(|x| x.id == id) else {
                                return "box".to_string();
                            };
                            if b.queued {
                                return "box is-queued".to_string();
                            }
                            match b.mode {
                                Mode::Moving => "box is-moving".to_string(),
                                Mode::Refusing { .. } => "box is-refusing".to_string(),
                                Mode::Idle => "box".to_string(),
                            }
                        };

                        let top_now = move || {
                            let live = boxes.get();
                            let y = live
                                .iter()
                                .find(|x| x.id == id)
                                .map(|x| x.y)
                                .unwrap_or(0.0);
                            format!("{}px", y)
                        };

                        let left_px = format!("{}px", BOX_X);
                        let w_px = format!("{}px", BOX_W);
                        let h_px = format!("{}px", BOX_H);

                        view! {
                            <div
                                class=class_now
                                style:left=left_px
                                style:top=top_now
                                style:width=w_px
                                style:height=h_px
                            >
                                <div
                                    class="box-region box-up"
                                    on:click=move |_| handle_click(&boxes, &click_queue, id, -1)
                                >
                                    <span class="arrow">"↑"</span>
                                </div>
                                <hr class="box-hr" />
                                <div
                                    class="box-region box-down"
                                    on:click=move |_| handle_click(&boxes, &click_queue, id, 1)
                                >
                                    <span class="arrow">"↓"</span>
                                </div>
                                <span class="box-label">{label}</span>
                            </div>
                        }
                    }
                />
            </div>
        </div>
    }
}