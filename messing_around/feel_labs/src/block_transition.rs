use leptos::{ev, html, prelude::*};
use stylist::style;
use wasm_bindgen::JsCast;

const RISE_DURATION_MS: u32 = 550;
const RISE_DISTANCE_PX: i32 = 28;

const SETTLE_DURATION_MS: u32 = 500;
const SETTLE_DISTANCE_PX: i32 = 22;

const INDENT_UNIT_PX: i32 = 20;

const INITIAL_SAMPLE: &str =
    "Intro\n    First point\n        Nested detail\n    Second point\nEnd";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Variant {
    SoftRise,
    IndentSettle,
}

impl Variant {
    fn anim_class(self) -> &'static str {
        match self {
            Variant::SoftRise => "bt-line-rise",
            Variant::IndentSettle => "bt-line-settle",
        }
    }
    fn label(self) -> &'static str {
        match self {
            Variant::SoftRise => "Soft rise",
            Variant::IndentSettle => "Indent settle",
        }
    }
}

#[derive(Clone, PartialEq)]
struct PreviewLine {
    index: usize,
    indent_level: usize,
    generation: u64,
}

fn scan_lines(text: &str, trigger_index: Option<usize>, generation: u64) -> Vec<PreviewLine> {
    text.split('\n')
        .enumerate()
        .map(|(i, line)| {
            let indent_level = line
                .chars()
                .take_while(|c| *c == ' ' || *c == '\t')
                .count();
            let is_animating = trigger_index == Some(i) && generation > 0;
            PreviewLine {
                index: i,
                indent_level,
                generation: if is_animating { generation } else { 0 },
            }
        })
        .collect()
}

#[component]
pub fn BlockTransition() -> AnyView {
    let styles = match style!(
        r#"
.bt-root {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 16px;
    --bt-rise-dur: ${rise_dur}ms;
    --bt-rise-dist: ${rise_dist}px;
    --bt-settle-dur: ${settle_dur}ms;
    --bt-settle-dist: ${settle_dist}px;
}

.bt-buttons {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
}

.bt-btn {
    font: inherit;
    font-size: 13px;
    padding: 6px 12px;
    border: 1px solid rgba(128, 128, 128, 0.5);
    border-radius: 6px;
    background: transparent;
    color: inherit;
    cursor: pointer;
}

.bt-btn:hover {
    border-color: rgba(128, 128, 128, 0.9);
}

.bt-input {
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 14px;
    line-height: 1.5;
    padding: 10px;
    min-height: 120px;
    resize: vertical;
    border: 1px solid rgba(128, 128, 128, 0.5);
    border-radius: 6px;
    background: rgba(128, 128, 128, 0.06);
    color: inherit;
}

.bt-previews {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
}

.bt-pane {
    border: 1px solid rgba(128, 128, 128, 0.4);
    border-radius: 6px;
    padding: 12px;
    min-height: 240px;
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 14px;
    line-height: 1.5;
}

.bt-pane-title {
    font-weight: 600;
    margin-bottom: 8px;
    font-family: system-ui, -apple-system, "Segoe UI", Roboto, sans-serif;
}

.bt-line, .bt-line-rise, .bt-line-settle {
    padding: 1px 0;
    white-space: pre;
    will-change: transform, opacity;
}

.bt-line-rise {
    animation: bt-soft-rise var(--bt-rise-dur, 550ms) cubic-bezier(0.22, 1, 0.36, 1) both;
}

.bt-line-settle {
    animation: bt-indent-settle var(--bt-settle-dur, 500ms) cubic-bezier(0.34, 1.56, 0.64, 1) both;
}

@keyframes bt-soft-rise {
    from { transform: translateY(var(--bt-rise-dist, 28px)); opacity: 0; }
    to   { transform: translateY(0); opacity: 1; }
}

@keyframes bt-indent-settle {
    0%   { transform: translateX(var(--bt-settle-dist, 22px)); opacity: 0.4; }
    60%  { transform: translateX(calc(var(--bt-settle-dist, 22px) * -0.25)); opacity: 1; }
    100% { transform: translateX(0); opacity: 1; }
}

@media (prefers-reduced-motion: reduce) {
    .bt-line-rise, .bt-line-settle {
        animation-duration: 1ms;
    }
}
"#,
        rise_dur = RISE_DURATION_MS,
        rise_dist = RISE_DISTANCE_PX,
        settle_dur = SETTLE_DURATION_MS,
        settle_dist = SETTLE_DISTANCE_PX,
    ) {
        Ok(s) => s,
        Err(e) => {
            leptos::logging::error!("block_transition: style build failed: {e}");
            return view! {
                <div>"Block transition styles failed to build. See console."</div>
            }
            .into_any();
        }
    };
    let root_class = styles.get_class_name().to_string();

    let text = RwSignal::new(INITIAL_SAMPLE.to_string());
    let generation = RwSignal::new(0u64);
    let trigger_line = RwSignal::new(0usize);
    let next_block = RwSignal::new(1u64);

    let textarea_ref = NodeRef::<html::Textarea>::new();

    Effect::new(move |_| {
        let Some(el) = textarea_ref.get() else {
            return;
        };
        let desired = text.get();
        if el.value() != desired {
            el.set_value(&desired);
        }
    });

    let on_input = move |ev: ev::Event| {
        let Some(target) = ev.target() else {
            leptos::logging::error!("block_transition: input event has no target");
            return;
        };
        let Ok(ta) = target.dyn_into::<web_sys::HtmlTextAreaElement>() else {
            leptos::logging::error!("block_transition: input target is not a textarea");
            return;
        };
        text.set(ta.value());
    };

    let add_block = move |_| {
        let n = next_block.get_untracked();
        next_block.set(n.wrapping_add(1));
        text.update(|t| {
            if !t.is_empty() && !t.ends_with('\n') {
                t.push('\n');
            }
            t.push_str(&format!("New block {n}"));
        });
        let idx = text.with_untracked(|t| t.split('\n').count().saturating_sub(1));
        trigger_line.set(idx);
        generation.update(|g| *g = g.wrapping_add(1));
    };

    let lines = Memo::new(move |_| {
        let t = text.get();
        let idx = trigger_line.get();
        let g = generation.get();
        scan_lines(&t, Some(idx), g)
    });

    let render_pane = move |variant: Variant| {
        let anim_class = variant.anim_class();
        view! {
            <For
                each=move || lines.get()
                key=|line| (line.index, line.generation)
                children=move |line: PreviewLine| {
                    let idx = line.index;
                    let indent_px = (line.indent_level as i32) * INDENT_UNIT_PX;
                    let class_attr = if line.generation > 0 { anim_class } else { "bt-line" };
                    let style_attr = format!("margin-left: {indent_px}px;");
                    let text_for_line = text;
                    view! {
                        <div class=class_attr style=style_attr>
                            {move || {
                                text_for_line.with(|t| {
                                    match t.split('\n').nth(idx) {
                                        Some(raw) => {
                                            let n = raw
                                                .chars()
                                                .take_while(|c| *c == ' ' || *c == '\t')
                                                .count();
                                            raw.chars().skip(n).collect::<String>()
                                        }
                                        None => String::new(),
                                    }
                                })
                            }}
                        </div>
                    }
                }
            />
        }
    };

    let pane_a = render_pane(Variant::SoftRise);
    let pane_b = render_pane(Variant::IndentSettle);
    let label_a = Variant::SoftRise.label();
    let label_b = Variant::IndentSettle.label();

    view! {
        <div class=root_class>
            <div class="bt-buttons">
                <button class="bt-btn" on:click=add_block>"Add block"</button>
            </div>
            <textarea
                node_ref=textarea_ref
                class="bt-input"
                spellcheck="false"
                on:input=on_input
            ></textarea>
            <div class="bt-previews">
                <div class="bt-pane">
                    <div class="bt-pane-title">{label_a}</div>
                    {pane_a}
                </div>
                <div class="bt-pane">
                    <div class="bt-pane-title">{label_b}</div>
                    {pane_b}
                </div>
            </div>
        </div>
    }
    .into_any()
}