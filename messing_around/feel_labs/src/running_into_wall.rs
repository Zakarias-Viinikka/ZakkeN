use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use std::time::Duration;

const PANE_LINE_HEIGHT_PX: f64 = 24.0;
const AUTO_TYPE_INTERVAL_MS: u64 = 130;

const AUTO_TYPE_WORDS: &[&str] = &[
    "The", "quick", "brown", "fox", "jumps", "over", "the", "lazy", "dog",
    "and", "then", "keeps", "running", "past", "the", "wall", "again", "and",
    "again", "and", "again", "until", "it", "wraps", "and", "wraps", "and",
    "wraps", "and", "keeps", "going", "past", "the", "edge", "every", "time",
];

fn measure_rows(node: &web_sys::HtmlDivElement) -> i32 {
    let h = node.client_height() as f64;
    ((h / PANE_LINE_HEIGHT_PX).round() as i32).max(0)
}

#[component]
pub fn RunningIntoWall() -> impl IntoView {
    let text = RwSignal::new(String::new());
    let measure_trigger = RwSignal::new(0u64);
    let rows_display = RwSignal::new(0i32);
    let bar = RwSignal::new(None::<(u64, i32)>);
    let bar_gen = RwSignal::new(0u64);
    let baseline = RwSignal::new(0i32);
    let baseline_ready = RwSignal::new(false);
    let word_idx = RwSignal::new(0usize);
    let auto_running = RwSignal::new(false);

    let pane_ref = NodeRef::<html::Div>::new();

    Effect::new(move |_| {
        let _ = measure_trigger.get();
        let Some(el) = pane_ref.get() else {
            return;
        };
        let rows_now = measure_rows(&el);
        rows_display.set(rows_now);

        if baseline_ready.get_untracked() {
            let before = baseline.get_untracked();
            if rows_now > before && before > 0 {
                bar_gen.update(|g| *g = g.wrapping_add(1));
                let g = bar_gen.get_untracked();
                let top_px = (before as f64 * PANE_LINE_HEIGHT_PX) as i32;
                bar.set(Some((g, top_px)));
            }
        }
        baseline.set(rows_now);
        baseline_ready.set(true);
    });

    Effect::new(move |_| {
        let handle = window_event_listener(ev::resize, move |_| {
            baseline_ready.set(false);
            measure_trigger.update(|n| *n = n.wrapping_add(1));
        });
        on_cleanup(move || handle.remove());
    });

    Effect::new(move |_| {
        if !auto_running.get() {
            return;
        }
        let h = set_interval_with_handle(
            move || {
                let idx = word_idx.get_untracked();
                let word = AUTO_TYPE_WORDS[idx % AUTO_TYPE_WORDS.len()];
                word_idx.set(idx.wrapping_add(1));
                text.update(|t| {
                    if !t.is_empty() {
                        t.push(' ');
                    }
                    t.push_str(word);
                });
                measure_trigger.update(|n| *n = n.wrapping_add(1));
            },
            Duration::from_millis(AUTO_TYPE_INTERVAL_MS),
        );
        match h {
            Ok(handle) => on_cleanup(move || handle.clear()),
            Err(e) => {
                leptos::logging::error!("running_into_wall: set interval: {e:?}");
            }
        }
    });

    let start_auto = move |_| auto_running.set(true);
    let stop_auto = move |_| auto_running.set(false);
    let reset = move |_| {
        auto_running.set(false);
        text.set(String::new());
        word_idx.set(0);
        baseline_ready.set(false);
        baseline.set(0);
        rows_display.set(0);
        bar.set(None);
        bar_gen.set(0);
        measure_trigger.update(|n| *n = n.wrapping_add(1));
    };

    view! {
        <div class="rtw-root">
            <div class="rtw-toolbar">
                <button class="rtw-btn" on:click=start_auto>"Type for me"</button>
                <button class="rtw-btn" on:click=stop_auto>"Stop"</button>
                <button class="rtw-btn" on:click=reset>"Reset"</button>
            </div>
            <div class="rtw-pane">
                <div class="rtw-pane-title">
                    <span>"Running into the wall"</span>
                    <span>{move || format!("rows: {}", rows_display.get())}</span>
                </div>
                <div class="rtw-text" node_ref=pane_ref>
                    {move || text.get()}
                    <For
                        each=move || {
                            bar.get().map(|entry| vec![entry]).unwrap_or_default()
                        }
                        key=|(g, _)| *g
                        children=move |(_, top)| {
                            let style_attr = format!("top: {top}px;");
                            view! { <div class="rtw-bar" style=style_attr></div> }
                        }
                    />
                </div>
            </div>
        </div>
    }
}
