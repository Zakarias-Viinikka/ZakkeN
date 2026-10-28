use leptos::ev;
use leptos::prelude::*;
use std::time::Duration;
use wasm_bindgen::JsCast;

const IDLE_THRESHOLD_MS: f64 = 2000.0;
const TICK_INTERVAL_MS: u64 = 100;

const PAGES: &[&str] = &[
    "Weekly planning",
    "Reading list",
    "Trip itinerary",
    "Recipes to try",
    "Meeting notes",
    "Budget 2026",
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Scene { StackReveal, Recede, Outline }

#[component]
pub fn IdlePage() -> impl IntoView {
    let variant = RwSignal::new(Scene::StackReveal);
    let is_idle = RwSignal::new(false);
    let last_activity = StoredValue::new(js_sys::Date::now());

    Effect::new(move |_| {
        let h = set_interval_with_handle(
            move || {
                let now = js_sys::Date::now();
                let last = last_activity.get_value();
                let should = (now - last) >= IDLE_THRESHOLD_MS;
                if should != is_idle.get_untracked() {
                    is_idle.set(should);
                }
            },
            Duration::from_millis(TICK_INTERVAL_MS),
        );
        match h {
            Ok(handle) => on_cleanup(move || handle.clear()),
            Err(e) => leptos::logging::error!("idle_page: tick: {e:?}"),
        }
    });

    Effect::new(move |_| {
        let last_pos = StoredValue::new((0.0_f64, 0.0_f64));
        let h1 = window_event_listener(ev::pointermove, move |e: ev::PointerEvent| {
            let x = e.client_x() as f64;
            let y = e.client_y() as f64;
            let (px, py) = last_pos.get_value();
            if (x - px).abs() < 0.5 && (y - py).abs() < 0.5 { return; }
            last_pos.set_value((x, y));
            last_activity.set_value(js_sys::Date::now());
            if is_idle.get_untracked() { is_idle.set(false); }
        });
        let h2 = window_event_listener(ev::pointerdown, move |_| {
            last_activity.set_value(js_sys::Date::now());
            if is_idle.get_untracked() { is_idle.set(false); }
        });
        let h3 = window_event_listener(ev::keydown, move |_| {
            last_activity.set_value(js_sys::Date::now());
            if is_idle.get_untracked() { is_idle.set(false); }
        });
        let h4 = window_event_listener(ev::scroll, move |_| {
            last_activity.set_value(js_sys::Date::now());
            if is_idle.get_untracked() { is_idle.set(false); }
        });
        on_cleanup(move || {
            h1.remove(); h2.remove(); h3.remove(); h4.remove();
        });
    });

    let root_class = move || if is_idle.get() { "ip-root idle" } else { "ip-root" };

    view! {
        <div class=root_class>
            <div class="ip-toolbar">
                <div class="seg">
                    <button class="seg-btn" class:active=move || variant.get() == Scene::StackReveal
                        on:click=move |_| variant.set(Scene::StackReveal)>"Stack reveal"</button>
                    <button class="seg-btn" class:active=move || variant.get() == Scene::Recede
                        on:click=move |_| variant.set(Scene::Recede)>"Recede"</button>
                    <button class="seg-btn" class:active=move || variant.get() == Scene::Outline
                        on:click=move |_| variant.set(Scene::Outline)>"Outline"</button>
                </div>
                <span class="label">{move || if is_idle.get() { "Idle" } else { "Active" }}</span>
            </div>
            {move || match variant.get() {
                Scene::StackReveal => view! { <SceneStack /> }.into_any(),
                Scene::Recede => view! { <SceneRecede /> }.into_any(),
                Scene::Outline => view! { <SceneOutline /> }.into_any(),
            }}
        </div>
    }
}

#[component]
fn SceneStack() -> impl IntoView {
    view! {
        <div class="ip-stage ip-stack-stage">
            <div class="ip-stack-frame">
                <div class="ip-stack-sheet ip-stack-sheet-3">
                    <div class="ip-sheet-inner">
                        <span class="label">"Journal"</span>
                        <div class="ip-skel-line"></div>
                        <div class="ip-skel-line short"></div>
                        <div class="ip-skel-line"></div>
                    </div>
                </div>
                <div class="ip-stack-sheet ip-stack-sheet-2">
                    <div class="ip-sheet-inner">
                        <span class="label">"Recipes to try"</span>
                        <div class="ip-skel-line"></div>
                        <div class="ip-skel-line short"></div>
                        <div class="ip-skel-line"></div>
                    </div>
                </div>
                <div class="ip-stack-sheet ip-stack-sheet-1">
                    <div class="ip-sheet-inner">
                        <span class="label">"Reading list"</span>
                        <div class="ip-skel-line"></div>
                        <div class="ip-skel-line short"></div>
                        <div class="ip-skel-line"></div>
                    </div>
                </div>
                <div class="ip-stack-top mock-page">
                    <div class="mock-title">"Weekly planning"</div>
                    <div class="mock-block">"Three things done, two pushed, one dropped on purpose."</div>
                    <div class="mock-block">"Next week: shorter list, fewer meetings."</div>
                    <div class="mock-block">"Keep the mornings free for deep work."</div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn SceneRecede() -> impl IntoView {
    view! {
        <div class="ip-stage ip-recede-stage">
            <div class="ip-recede-app">
                <aside class="ip-recede-sidebar">
                    {PAGES.iter().enumerate().map(|(i, name)| {
                        let active = i == 0;
                        let class = if active { "ip-recede-nav ip-recede-nav-active" } else { "ip-recede-nav" };
                        view! { <div class=class>{*name}</div> }
                    }).collect_view()}
                </aside>
                <div class="ip-recede-main">
                    <header class="ip-recede-topbar">
                        <span class="label">"Pages / Weekly planning"</span>
                        <span class="ip-recede-dot"></span>
                        <span class="ip-recede-dot"></span>
                    </header>
                    <div class="mock-page ip-recede-page">
                        <div class="mock-title">"Weekly planning"</div>
                        <div class="mock-block">"Three things done, two pushed, one dropped on purpose."</div>
                        <div class="mock-block">"Next week: shorter list, fewer meetings."</div>
                        <div class="mock-block">"Keep the mornings free for deep work."</div>
                        <div class="mock-block">"Pick up the parcel on Friday."</div>
                    </div>
                </div>
            </div>
            <div class="ip-recede-vignette"></div>
        </div>
    }
}

#[component]
fn SceneOutline() -> impl IntoView {
    let blocks: &[(&str, usize)] = &[
        ("Three things done, two pushed, one dropped on purpose.", 3),
        ("Next week: shorter list, fewer meetings.", 2),
        ("Keep the mornings free for deep work.", 2),
        ("Pick up the parcel on Friday.", 2),
    ];
    view! {
        <div class="ip-stage ip-outline-stage">
            <div class="mock-page ip-outline-page">
                <div class="mock-title ip-outline-block" style="--b: 0">
                    <span class="ip-outline-text">"Weekly planning"</span>
                    <span class="ip-outline-bars">
                        <span class="ip-outline-bar" style="--i: 0"></span>
                        <span class="ip-outline-bar" style="--i: 1"></span>
                    </span>
                </div>
                {blocks.iter().enumerate().map(|(bi, (text, n))| {
                    let bars = (0..*n).map(|i| {
                        view! { <span class="ip-outline-bar" style=format!("--i: {}", i)></span> }
                    }).collect_view();
                    view! {
                        <div class="mock-block ip-outline-block" style=format!("--b: {}", bi + 1)>
                            <span class="ip-outline-text">{*text}</span>
                            <span class="ip-outline-bars">{bars}</span>
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}