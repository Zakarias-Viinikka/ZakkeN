use leptos::prelude::*;
use std::time::Duration;
use wasm_bindgen::JsCast;

// Timeline (ms from Confirm click):
//   0    - 120   anticipate: slot pop, flash rises
//   120          draft hidden; tiles mounted (were over the draft already)
//   120  - 880   tiles fly outward (TILE_MS + stagger, centre first)
//   120  - 540   settled page reveals (opacity over 300, scale over 420)
//   120  - 720   ring shockwave follows page edge
//   620  - 1420  shine sweeps across the settled page
//   1500         phase Done; tiles/ring/flash unmount, settled remains

const TILE_COLS: usize = 6;
const TILE_ROWS: usize = 4;
const TILE_MS: u32 = 640;
const SHATTER_AT: u32 = 120;
const TILE_STAGGER: u32 = 120;
const CONFIRM_TOTAL_MS: u32 = 1500;
const REDUCED_TOTAL_MS: u32 = 320;

const INITIAL_A: &[&str] = &[
    "Friday: arrive after lunch, check in, take the long way round the shoreline before dinner.",
    "Saturday: slow breakfast, a short hike if the weather holds, nothing booked until the evening.",
    "Sunday: pack up by eleven, stop for coffee on the way back, home before the traffic builds.",
    "Bring: the paperback, the good boots, a thermos.",
];

const INITIAL_B: &[&str] = &[
    "Friday 14:10 train out, check in by 16:00, dinner reservation at 19:30.",
    "Saturday 08:00 summit trail (five hours round trip), packed lunch, back for a late swim.",
    "Saturday 20:00 tasting menu at the village restaurant, booked for two.",
    "Sunday 09:30 rowing lesson, then the market before the noon train.",
    "Bring: rain shell, headlamp, printed tickets, cash for the market.",
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Side { A, B }

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase { Pick, Edit, Confirming, Done }

fn init_a() -> Vec<String> { INITIAL_A.iter().map(|s| s.to_string()).collect() }
fn init_b() -> Vec<String> { INITIAL_B.iter().map(|s| s.to_string()).collect() }

fn prefers_reduced_motion() -> bool {
    let Some(win) = web_sys::window() else { return false; };
    match win.match_media("(prefers-reduced-motion: reduce)") {
        Ok(Some(m)) => m.matches(),
        Ok(None) => false,
        Err(_) => false,
    }
}

fn hash_u32(x: u32) -> u32 {
    let mut h = x;
    h ^= h >> 16;
    h = h.wrapping_mul(0x7feb352d);
    h ^= h >> 15;
    h = h.wrapping_mul(0x846ca68b);
    h ^= h >> 16;
    h
}

struct TileParams {
    dx: f64, dy: f64, rot: f64, delay: u32,
    origin_x: f64, origin_y: f64,
    clip_top: f64, clip_right: f64, clip_bottom: f64, clip_left: f64,
}

fn tile_params(r: usize, c: usize) -> TileParams {
    let cols = TILE_COLS as f64;
    let rows = TILE_ROWS as f64;
    let cx = (c as f64 + 0.5) / cols;
    let cy = (r as f64 + 0.5) / rows;
    let ddx = cx - 0.5;
    let ddy = cy - 0.5;
    let dist = (ddx * ddx + ddy * ddy).sqrt();
    let dist_norm = dist / 0.707;
    let nx = if dist > 0.001 { ddx / dist } else { 0.0 };
    let ny = if dist > 0.001 { ddy / dist } else { 0.0 };
    let jitter = (hash_u32((r * 1000 + c) as u32) as f64) / (u32::MAX as f64);
    let mag = 120.0 + dist_norm * 260.0 + (jitter - 0.5) * 60.0;
    let dx = nx * mag;
    let dy = ny * mag + 50.0;
    let rot = (jitter - 0.5) * 52.0;
    let delay = SHATTER_AT + (dist_norm * TILE_STAGGER as f64) as u32;
    TileParams {
        dx, dy, rot, delay,
        origin_x: cx * 100.0,
        origin_y: cy * 100.0,
        clip_top: (r as f64 / rows) * 100.0,
        clip_right: ((TILE_COLS - 1 - c) as f64 / cols) * 100.0,
        clip_bottom: ((TILE_ROWS - 1 - r) as f64 / rows) * 100.0,
        clip_left: (c as f64 / cols) * 100.0,
    }
}

#[component]
pub fn TwoVersion() -> impl IntoView {
    let reduced = prefers_reduced_motion();
    let phase = RwSignal::new(Phase::Pick);
    let picked = RwSignal::new(None::<Side>);
    let texts_a = RwSignal::new(init_a());
    let texts_b = RwSignal::new(init_b());
    let board_gen = RwSignal::new(0u64);
    let confirm_gen = RwSignal::new(0u64);
    let timer: RwSignal<Option<TimeoutHandle>> = RwSignal::new(None);

    on_cleanup(move || {
        if let Some(h) = timer.get_untracked() { h.clear(); }
    });

    let pick_a = move |_| {
        if phase.get_untracked() == Phase::Pick {
            picked.set(Some(Side::A));
            phase.set(Phase::Edit);
        }
    };
    let pick_b = move |_| {
        if phase.get_untracked() == Phase::Pick {
            picked.set(Some(Side::B));
            phase.set(Phase::Edit);
        }
    };

    let change_pick = move |_| {
        if phase.get_untracked() == Phase::Edit {
            picked.set(None);
            phase.set(Phase::Pick);
        }
    };

    let reset = move |_| {
        if let Some(h) = timer.get_untracked() { h.clear(); }
        timer.set(None);
        phase.set(Phase::Pick);
        picked.set(None);
        texts_a.set(init_a());
        texts_b.set(init_b());
        board_gen.update(|g| *g += 1);
    };

    let start_confirm = move |_| {
        if phase.get_untracked() != Phase::Edit { return; }
        phase.set(Phase::Confirming);
        confirm_gen.update(|g| *g += 1);
        let total = if reduced { REDUCED_TOTAL_MS } else { CONFIRM_TOTAL_MS };
        let h = set_timeout_with_handle(
            move || phase.set(Phase::Done),
            Duration::from_millis(total as u64),
        );
        match h {
            Ok(handle) => timer.set(Some(handle)),
            Err(e) => leptos::logging::error!("two_version: confirm timer: {e:?}"),
        }
    };

    let board_data_pick = move || match (phase.get(), picked.get()) {
        (Phase::Pick, _) | (_, None) => "none",
        (_, Some(Side::A)) => "a",
        (_, Some(Side::B)) => "b",
    };
    let board_phase_class = move || {
        let mut c = String::from("tv-pair");
        match phase.get() {
            Phase::Pick => {}
            Phase::Edit => c.push_str(" editing"),
            Phase::Confirming => c.push_str(" confirming"),
            Phase::Done => c.push_str(" done"),
        }
        c
    };

    let step_class = move |n: u32| {
        let p = match phase.get() {
            Phase::Pick => 1,
            Phase::Edit => 2,
            Phase::Confirming | Phase::Done => 3,
        };
        if n < p { "tv-step done" }
        else if n == p { "tv-step active" }
        else { "tv-step" }
    };

    view! {
        <div class="tv-root">
            <div class="tv-toolbar">
                <div class="tv-steps">
                    <span class=move || step_class(1)>
                        <span class="tv-step-num">{move || if matches!(phase.get(), Phase::Pick) { "1" } else { "\u{2713}" }}</span>
                        " Pick"
                    </span>
                    <span class=move || step_class(2)>
                        <span class="tv-step-num">{move || match phase.get() { Phase::Pick => "2", Phase::Edit => "2", _ => "\u{2713}" }}</span>
                        " Edit"
                    </span>
                    <span class=move || step_class(3)>
                        <span class="tv-step-num">"3"</span>
                        " Confirm"
                    </span>
                </div>
                <div class="tv-actions">
                    <button class="btn" on:click=reset>"Reset"</button>
                    {move || if phase.get() == Phase::Edit {
                        view! {
                            <button class="btn" on:click=change_pick>"Change pick"</button>
                            <button class="btn btn-accent" on:click=start_confirm>"Confirm"</button>
                        }.into_any()
                    } else {
                        view! { <span></span> }.into_any()
                    }}
                </div>
            </div>
            <For
                each=move || vec![board_gen.get()]
                key=|g| *g
                children=move |_| {
                    let texts_a = texts_a;
                    let texts_b = texts_b;
                    view! {
                        <div class=board_phase_class data-picked=board_data_pick>
                            <div class="tv-cell" class:is-picked=move || picked.get() == Some(Side::A)
                                class:is-loser=move || picked.get() == Some(Side::B)
                                on:click=pick_a>
                                {render_cell(Side::A, "Slow weekend", texts_a, phase, picked, confirm_gen, reduced)}
                            </div>
                            <div class="tv-cell" class:is-picked=move || picked.get() == Some(Side::B)
                                class:is-loser=move || picked.get() == Some(Side::A)
                                on:click=pick_b>
                                {render_cell(Side::B, "Packed weekend", texts_b, phase, picked, confirm_gen, reduced)}
                            </div>
                        </div>
                    }
                }
            />
        </div>
    }
}

fn render_cell(
    side: Side,
    version_name: &'static str,
    texts: RwSignal<Vec<String>>,
    phase: RwSignal<Phase>,
    picked: RwSignal<Option<Side>>,
    confirm_gen: RwSignal<u64>,
    reduced: bool,
) -> AnyView {
    let is_picked = move || picked.get() == Some(side);
    let is_editable = move || is_picked() && phase.get() == Phase::Edit;
    let is_confirming = move || is_picked() && phase.get() == Phase::Confirming;
    let is_done = move || is_picked() && phase.get() == Phase::Done;
    let draft_hidden = move || is_confirming() || is_done();

    let chip_text = move || {
        match phase.get() {
            Phase::Pick => if side == Side::A { "Version A" } else { "Version B" },
            Phase::Edit => if is_picked() { "Editing" } else { "" },
            Phase::Confirming => "",
            Phase::Done => if is_picked() { "Confirmed" } else { "" },
        }
    };

    let draft_view = render_draft(texts, version_name, chip_text, is_editable);

    let settled_view = move || {
        if !is_picked() || !(is_confirming() || is_done()) { return None; }
        let snap = texts.get_untracked();
        Some(static_face_view(&snap, version_name, "Confirmed"))
    };

    let tiles_view = move || {
        if !is_confirming() || reduced { return None; }
        let snap = texts.get_untracked();
        let tiles = (0..TILE_ROWS).flat_map(|r| {
            (0..TILE_COLS).map(move |c| (r, c))
        }).map(|(r, c)| render_tile(r, c, snap.clone(), version_name)).collect_view();
        Some(view! { <div class="tv-tiles">{tiles}</div> }.into_any())
    };

    let flash_view = move || {
        if !is_confirming() || reduced { return None; }
        Some(view! { <div class="tv-flash"></div> }.into_any())
    };
    let ring_view = move || {
        if !is_confirming() || reduced { return None; }
        Some(view! { <div class="tv-ring"></div> }.into_any())
    };

    view! {
        <div class="tv-slot">
            <div class="tv-draft" class:draft-hidden=draft_hidden>
                {draft_view}
            </div>
            {settled_view}
            {tiles_view}
            {flash_view}
            {ring_view}
            {move || if is_picked() && phase.get() == Phase::Confirming && !reduced {
                let _ = confirm_gen.get();
                None::<AnyView>
            } else { None }}
        </div>
    }.into_any()
}

fn render_draft(
    texts: RwSignal<Vec<String>>,
    version_name: &'static str,
    chip_text: impl Fn() -> &'static str + Copy + Send + Sync + 'static,
    is_editable: impl Fn() -> bool + Copy + Send + Sync + 'static,
) -> AnyView {
    let blocks = move || {
        let n = texts.with(|v| v.len());
        (0..n).collect::<Vec<usize>>()
    };
    view! {
        <div class="tv-face">
            <div class="tv-face-head">
                <span class="tv-version-name">{version_name}</span>
                <span class="tv-chip">{move || chip_text()}</span>
            </div>
            <h2 class="tv-page-title">"Weekend away"</h2>
            <div class="tv-blocks">
                <For
                    each=blocks
                    key=|i| *i
                    children=move |i: usize| {
                        let initial = texts.with_untracked(|v| v.get(i).cloned().unwrap_or_default());
                        let style = format!("--i: {}", i);
                        let texts_mirror = texts;
                        let texts_input = texts;
                        view! {
                            <div class="tv-block-live" style=style>
                                <div class="tv-text">
                                    {move || texts_mirror.with(|v| v.get(i).cloned().unwrap_or_default())}
                                    "\n"
                                </div>
                                <textarea
                                    class="tv-textarea"
                                    prop:value=initial
                                    prop:readonly=move || !is_editable()
                                    tabindex=move || if is_editable() { "0" } else { "-1" }
                                    on:input=move |ev| {
                                        if let Some(t) = ev.target() {
                                            if let Ok(ta) = t.dyn_into::<web_sys::HtmlTextAreaElement>() {
                                                let v = ta.value();
                                                texts_input.update(|list| {
                                                    if i < list.len() { list[i] = v; }
                                                });
                                            }
                                        }
                                    }
                                ></textarea>
                            </div>
                        }
                    }
                />
            </div>
        </div>
    }.into_any()
}

fn static_face_view(texts: &[String], version_name: &'static str, chip: &'static str) -> AnyView {
    let blocks = texts.iter().enumerate().map(|(i, t)| {
        let style = format!("--i: {}", i);
        let text = t.clone();
        view! {
            <div class="tv-block-static" style=style>
                <div class="tv-text-static">{text}</div>
            </div>
        }.into_any()
    }).collect_view();
    view! {
        <div class="tv-face">
            <div class="tv-face-head">
                <span class="tv-version-name">{version_name}</span>
                <span class="tv-chip">{chip}</span>
            </div>
            <h2 class="tv-page-title">"Weekend away"</h2>
            <div class="tv-blocks">{blocks}</div>
        </div>
    }.into_any()
}

fn render_tile(r: usize, c: usize, texts: Vec<String>, version_name: &'static str) -> AnyView {
    let p = tile_params(r, c);
    let style = format!(
        "--dx: {:.2}px; --dy: {:.2}px; --rot: {:.3}deg; --tv-tile-ms: {}ms; --tv-tile-delay: {}ms; --ox: {:.3}%; --oy: {:.3}%;",
        p.dx, p.dy, p.rot, TILE_MS, p.delay, p.origin_x, p.origin_y
    );
    let clip = format!(
        "clip-path: inset({:.4}% {:.4}% {:.4}% {:.4}%);",
        p.clip_top, p.clip_right, p.clip_bottom, p.clip_left
    );
    let face = static_face_view(&texts, version_name, "Editing");
    view! {
        <div class="tv-tile" style=style>
            <div class="tv-tile-clip" style=clip>
                {face}
            </div>
        </div>
    }.into_any()
}