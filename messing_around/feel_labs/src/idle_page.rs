use leptos::ev;
use leptos::prelude::*;
use std::time::Duration;

const IDLE_THRESHOLD_MS: u64 = 2000;
const TICK_INTERVAL_MS: u64 = 100;

// (x offset, y offset, tone, sway delay)
const UNDER_SHEETS: &[(i32, i32, &str, u32)] = &[
    (0, 0, "#ffffff", 0),
    (14, 6, "#f1f4f9", 300),
    (28, 12, "#e3e8f0", 600),
    (42, 18, "#d5dbe6", 900),
];

fn prefers_reduced_motion() -> bool {
    let Some(win) = web_sys::window() else {
        return false;
    };
    match win.match_media("(prefers-reduced-motion: reduce)") {
        Ok(Some(m)) => m.matches(),
        Ok(None) => false,
        Err(e) => {
            leptos::logging::error!("idle_page: match_media: {e:?}");
            false
        }
    }
}

#[component]
pub fn IdlePage() -> impl IntoView {
    let elapsed = RwSignal::new(0u64);
    let is_idle = RwSignal::new(false);

    let reduced = prefers_reduced_motion();

    Effect::new(move |_| {
        let h = set_interval_with_handle(
            move || {
                if is_idle.get_untracked() {
                    return;
                }
                let next = elapsed.get_untracked() + TICK_INTERVAL_MS;
                elapsed.set(next);
                if next >= IDLE_THRESHOLD_MS {
                    is_idle.set(true);
                }
            },
            Duration::from_millis(TICK_INTERVAL_MS),
        );
        match h {
            Ok(handle) => on_cleanup(move || handle.clear()),
            Err(e) => {
                leptos::logging::error!("idle_page: set interval: {e:?}");
            }
        }
    });

    let reset = move || {
        elapsed.set(0);
        if is_idle.get_untracked() {
            is_idle.set(false);
        }
    };
    let reset = StoredValue::new(reset);

    Effect::new(move |_| {
        let h1 = window_event_listener(ev::pointermove, move |_| reset.get_value()());
        let h2 = window_event_listener(ev::pointerdown, move |_| reset.get_value()());
        let h3 = window_event_listener(ev::keydown, move |_| reset.get_value()());
        let h4 = window_event_listener(ev::scroll, move |_| reset.get_value()());
        on_cleanup(move || {
            h1.remove();
            h2.remove();
            h3.remove();
            h4.remove();
        });
    });

    let bg_class = move || {
        if is_idle.get() {
            "ip-bg idle"
        } else {
            "ip-bg"
        }
    };

    let under_sheets = UNDER_SHEETS
        .iter()
        .map(|&(x, y, tone, delay)| {
            let style_attr = if reduced {
                format!("translate: {x}px {y}px; background: {tone};")
            } else {
                format!(
                    "translate: {x}px {y}px; background: {tone}; animation-delay: {delay}ms;"
                )
            };
            view! { <div class="ip-sheet ip-sheet-under" style=style_attr></div> }
        })
        .collect_view();

    view! {
        <div class="ip-root">
            <div class=bg_class>
                <div class="ip-sheet ip-sheet-top"></div>
                {under_sheets}
            </div>
            <div class="ip-content">
                <h2>"Sample page content"</h2>
                <p>"This text stays exactly where it is. Only the background moves."</p>
                <p>
                    "Leave the pointer alone for a couple of seconds. The page surface slides \
                     left, revealing the sheets stacked under it on the right."
                </p>
                <p>
                    "The revealed sheets sway gently. Move the pointer, press a key, or \
                     scroll and the surface slides back."
                </p>
            </div>
        </div>
    }
}