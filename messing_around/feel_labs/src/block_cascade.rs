use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use std::time::Duration;
use wasm_bindgen::JsCast;

const ACCEL: f32 = 1500.0;
const MAX_V: f32 = 600.0;
const DT: f32 = 0.016;
const SPRING_K: f32 = 380.0;
const SPRING_C: f32 = 22.0;
const SPRING_SETTLE_POS: f32 = 0.2;
const SPRING_SETTLE_VEL: f32 = 5.0;
const SPRING_MAX_V: f32 = 1200.0;
const RIPPLE_STAGGER_MS: u32 = 28;
const GAP: f32 = 8.0;
const TOP_PAD: f32 = 12.0;
const BOTTOM_PAD: f32 = 24.0;
const PAGE_W: f32 = 560.0;
const DELETE_FADE_MS: u64 = 140;

const INITIAL_TEXTS: &[(&str, f32)] = &[
    ("Booked the early train. The afternoon is open for the museum.", 60.0),
    ("Ask about the annual plan — it saves a bit over monthly.", 44.0),
    ("Two people for dinner on Friday, not three.", 44.0),
    ("The schema migration is the risky part; test on staging first.", 60.0),
    ("Short walk after lunch if the weather holds.", 44.0),
    ("Renew the library card before the end of the month.", 60.0),
];

const TEXT_POOL: &[&str] = &[
    "Pull the coffee order back by ten minutes.",
    "Cross-check the invoice against the bank statement.",
    "Move the meeting to Thursday morning.",
    "Look up the train times for Saturday.",
    "Write down the wifi password somewhere sensible.",
    "Pack the charger and the small notebook.",
    "Reply to the email from the design review.",
    "Make a note to try the new bakery on the corner.",
    "The follow-up call can wait until Monday.",
    "Print the return label before the courier arrives.",
];

const HEIGHT_CYCLE: &[f32] = &[44.0, 60.0, 84.0];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Variant {
    Fall,
    Spring,
    Ripple,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Idle,
    Moving,
    Dying,
}

#[derive(Clone)]
struct Body {
    id: u64,
    text: String,
    h: f32,
    y: f32,
    target: f32,
    vel: f32,
    delay: u32,
    phase: Phase,
    dying: bool,
}

fn initial_bodies() -> Vec<Body> {
    let mut y = TOP_PAD;
    INITIAL_TEXTS
        .iter()
        .enumerate()
        .map(|(i, (t, h))| {
            let b = Body {
                id: (i + 1) as u64,
                text: (*t).to_string(),
                h: *h,
                y,
                target: y,
                vel: 0.0,
                delay: 0,
                phase: Phase::Idle,
                dying: false,
            };
            y += *h + GAP;
            b
        })
        .collect()
}

fn layout(bodies: &mut Vec<Body>, variant: Variant, snap_ids: &[u64], reduced: bool) {
    let mut y = TOP_PAD;
    let mut rank = 0u32;
    for b in bodies.iter_mut() {
        if b.dying {
            continue;
        }
        b.target = y;
        let diff = (b.y - b.target).abs();
        let is_snap = reduced || snap_ids.contains(&b.id) || diff < 0.5;
        if is_snap {
            b.y = b.target;
            b.vel = 0.0;
            b.phase = Phase::Idle;
            b.delay = 0;
        } else {
            if b.phase == Phase::Idle {
                b.phase = Phase::Moving;
                b.delay = if variant == Variant::Ripple {
                    rank * RIPPLE_STAGGER_MS
                } else {
                    0
                };
            }
            rank = rank.wrapping_add(1);
        }
        y = b.target + b.h + GAP;
    }
}

fn step(bodies: &mut Vec<Body>, variant: Variant) -> bool {
    let mut did_work = false;
    let spring = matches!(variant, Variant::Spring | Variant::Ripple);
    let step_delay_ms = ((DT * 1000.0) as u32).max(1);
    for b in bodies.iter_mut() {
        if b.phase != Phase::Moving {
            continue;
        }
        did_work = true;
        if variant == Variant::Ripple && b.delay > 0 {
            b.delay = b.delay.saturating_sub(step_delay_ms);
            continue;
        }
        if spring {
            let disp = b.y - b.target;
            let accel = -SPRING_K * disp - SPRING_C * b.vel;
            b.vel += accel * DT;
            if b.vel > SPRING_MAX_V {
                b.vel = SPRING_MAX_V;
            } else if b.vel < -SPRING_MAX_V {
                b.vel = -SPRING_MAX_V;
            }
            b.y += b.vel * DT;
            if disp.abs() < SPRING_SETTLE_POS && b.vel.abs() < SPRING_SETTLE_VEL {
                b.y = b.target;
                b.vel = 0.0;
                b.phase = Phase::Idle;
            }
        } else {
            // Fall — mirrors drag_reorder_lab/cascade.rs:
            // constant acceleration, velocity capped, hard stop at target.
            let dir = (b.target - b.y).signum();
            b.vel = (b.vel + ACCEL * DT).min(MAX_V);
            b.y += dir * b.vel * DT;
            let reached = (dir > 0.0 && b.y >= b.target) || (dir < 0.0 && b.y <= b.target);
            if reached {
                b.y = b.target;
                b.vel = 0.0;
                b.phase = Phase::Idle;
            }
        }
    }
    did_work
}

fn prefers_reduced_motion() -> bool {
    let Some(win) = web_sys::window() else {
        return false;
    };
    match win.match_media("(prefers-reduced-motion: reduce)") {
        Ok(Some(m)) => m.matches(),
        Ok(None) => false,
        Err(e) => {
            leptos::logging::error!("block_cascade: match_media: {e:?}");
            false
        }
    }
}

fn page_height(bodies: &[Body]) -> f32 {
    let mut bottom = TOP_PAD;
    for b in bodies {
        let candidate = b.y.max(b.target) + b.h;
        if candidate > bottom {
            bottom = candidate;
        }
    }
    bottom + BOTTOM_PAD
}

#[component]
pub fn BlockCascade() -> impl IntoView {
    let reduced = prefers_reduced_motion();
    let variant = RwSignal::new(Variant::Fall);
    let bodies = RwSignal::new(initial_bodies());
    let next_id = RwSignal::new(100u64);
    let text_idx = RwSignal::new(0u64);
    let timeouts: RwSignal<Vec<TimeoutHandle>> = RwSignal::new(Vec::new());

    // Tick — same shape as the drag lab's black box: only `set` when moved.
    Effect::new(move |_| {
        let h = set_interval_with_handle(
            move || {
                let v = variant.get_untracked();
                let mut bs = bodies.get_untracked();
                if step(&mut bs, v) {
                    bodies.set(bs);
                }
            },
            Duration::from_millis(16),
        );
        match h {
            Ok(handle) => on_cleanup(move || handle.clear()),
            Err(e) => leptos::logging::error!("block_cascade: set interval: {e:?}"),
        }
    });

    on_cleanup(move || {
        let list = timeouts.get_untracked();
        for h in list {
            h.clear();
        }
    });

    let on_delete = Callback::new(move |id: u64| {
        let non_dying = bodies.with_untracked(|bs| bs.iter().filter(|b| !b.dying).count());
        if non_dying <= 1 {
            return;
        }
        bodies.update(|bs| {
            if let Some(b) = bs.iter_mut().find(|b| b.id == id) {
                b.dying = true;
                b.phase = Phase::Dying;
            }
            layout(bs, variant.get_untracked(), &[], reduced);
        });
        let h = set_timeout_with_handle(
            move || {
                bodies.update(|bs| bs.retain(|b| b.id != id));
            },
            Duration::from_millis(DELETE_FADE_MS),
        );
        match h {
            Ok(handle) => timeouts.update(|v| v.push(handle)),
            Err(e) => leptos::logging::error!("block_cascade: delete timeout: {e:?}"),
        }
    });

    let on_insert = Callback::new(move |id: u64| {
        let new_id = next_id.get_untracked();
        next_id.set(new_id.wrapping_add(1));
        let idx = text_idx.get_untracked();
        text_idx.set(idx.wrapping_add(1));
        let text = TEXT_POOL[idx as usize % TEXT_POOL.len()].to_string();
        let h = HEIGHT_CYCLE[idx as usize % HEIGHT_CYCLE.len()];
        bodies.update(|bs| {
            let Some(pos) = bs.iter().position(|b| b.id == id) else {
                return;
            };
            bs.insert(
                pos + 1,
                Body {
                    id: new_id,
                    text,
                    h,
                    y: 0.0,
                    target: 0.0,
                    vel: 0.0,
                    delay: 0,
                    phase: Phase::Idle,
                    dying: false,
                },
            );
            layout(bs, variant.get_untracked(), &[new_id], reduced);
        });
    });

    let on_reset = move |_| {
        let list = timeouts.get_untracked();
        for h in list {
            h.clear();
        }
        timeouts.set(Vec::new());
        next_id.set(100);
        text_idx.set(0);
        bodies.set(initial_bodies());
    };

    let page_height_style = move || {
        let h = bodies.with(|bs| page_height(bs));
        format!("height: {}px;", h)
    };

    view! {
        <div class="cl-root">
            <div class="cl-toolbar">
                <div class="seg">
                    <button
                        class="seg-btn"
                        class:active=move || variant.get() == Variant::Fall
                        on:click=move |_| variant.set(Variant::Fall)
                    >"Fall"</button>
                    <button
                        class="seg-btn"
                        class:active=move || variant.get() == Variant::Spring
                        on:click=move |_| variant.set(Variant::Spring)
                    >"Spring"</button>
                    <button
                        class="seg-btn"
                        class:active=move || variant.get() == Variant::Ripple
                        on:click=move |_| variant.set(Variant::Ripple)
                    >"Ripple spring"</button>
                </div>
                <button class="btn" on:click=on_reset>"Reset"</button>
            </div>
            <div class="cl-page mock-page" style=format!("width: {}px;", PAGE_W)>
                <div class="cl-title">"Field notes"</div>
                <div class="cl-body" style=page_height_style>
                    <For
                        each=move || bodies.get()
                        key=|b| b.id
                        children=move |b: Body| {
                            render_block(b, bodies, on_delete, on_insert)
                        }
                    />
                </div>
            </div>
        </div>
    }
}

fn render_block(
    b: Body,
    bodies: RwSignal<Vec<Body>>,
    on_delete: Callback<u64>,
    on_insert: Callback<u64>,
) -> impl IntoView {
    let id = b.id;
    let initial_text = b.text.clone();
    let block_h = b.h;
    let textarea_ref = NodeRef::<html::Textarea>::new();

    let top_style = move || {
        let y = bodies.with(|bs| {
            bs.iter()
                .find(|x| x.id == id)
                .map(|x| x.y)
                .unwrap_or(0.0)
        });
        format!("transform: translateY({}px);", y)
    };
    let height_style = format!("height: {}px;", block_h);
    let class = move || {
        let dying = bodies.with(|bs| {
            bs.iter()
                .find(|x| x.id == id)
                .map(|x| x.dying)
                .unwrap_or(false)
        });
        if dying {
            "cl-block dying"
        } else {
            "cl-block"
        }
    };

    // Sync external text changes into the textarea (only fires on reset,
    // since typing updates bodies with the same value the textarea already
    // shows). Preserves caret when the value matches — which is what typing
    // does — so no flicker during editing.
    let text_signal = Memo::new(move |_| {
        bodies.with(|bs| {
            bs.iter()
                .find(|x| x.id == id)
                .map(|x| x.text.clone())
                .unwrap_or_default()
        })
    });
    Effect::new(move |_| {
        let new_val = text_signal.get();
        let Some(el) = textarea_ref.get_untracked() else {
            return;
        };
        if el.value() != new_val {
            el.set_value(&new_val);
        }
    });

    let on_input = move |ev: ev::Event| {
        let Some(t) = ev.target() else {
            return;
        };
        let Ok(ta) = t.dyn_into::<web_sys::HtmlTextAreaElement>() else {
            return;
        };
        let v = ta.value();
        bodies.update(|bs| {
            if let Some(bb) = bs.iter_mut().find(|x| x.id == id) {
                bb.text = v;
            }
        });
    };

    let on_del = move |_| on_delete.run(id);
    let on_ins = move |_| on_insert.run(id);

    view! {
        <div class=class style=top_style>
            <div class="cl-block-inner" style=height_style>
                <textarea
                    node_ref=textarea_ref
                    class="cl-text"
                    prop:value=initial_text
                    on:input=on_input
                    rows="1"
                    spellcheck="false"
                ></textarea>
                <div class="cl-actions">
                    <button
                        class="cl-action cl-delete"
                        aria-label="Delete block"
                        on:click=on_del
                    >"\u{00d7}"</button>
                    <button
                        class="cl-action cl-insert"
                        aria-label="Insert block below"
                        on:click=on_ins
                    >"+"</button>
                </div>
            </div>
        </div>
    }
}