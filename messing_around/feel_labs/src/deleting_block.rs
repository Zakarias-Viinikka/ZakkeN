use leptos::prelude::*;
use std::time::Duration;

const RED_MARK_MS: u32 = 400;
const RED_LEAVE_MS: u32 = 320;
const STRIKE_MARK_MS: u32 = 380;
const STRIKE_LEAVE_MS: u32 = 340;
const DROP_MARK_MS: u32 = 200;
const DROP_LEAVE_MS: u32 = 300;

const REDUCED_MARK_MS: u32 = 120;
const REDUCED_LEAVE_MS: u32 = 1;
const REMOVE_MARGIN_MS: u32 = 40;

const INITIAL_BLOCKS: &[&str] = &[
    "First block of text.",
    "Second block of text.",
    "Third block of text.",
    "Fourth block of text.",
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Variant { RedFade, StrikeSlide, Drop }

impl Variant {
    fn attr(self) -> &'static str {
        match self {
            Variant::RedFade => "red",
            Variant::StrikeSlide => "strike",
            Variant::Drop => "drop",
        }
    }
    fn label(self) -> &'static str {
        match self {
            Variant::RedFade => "Red fade",
            Variant::StrikeSlide => "Strike & slide",
            Variant::Drop => "Drop",
        }
    }
    fn mark_ms(self) -> u32 {
        match self {
            Variant::RedFade => RED_MARK_MS,
            Variant::StrikeSlide => STRIKE_MARK_MS,
            Variant::Drop => DROP_MARK_MS,
        }
    }
    fn leave_ms(self) -> u32 {
        match self {
            Variant::RedFade => RED_LEAVE_MS,
            Variant::StrikeSlide => STRIKE_LEAVE_MS,
            Variant::Drop => DROP_LEAVE_MS,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase { Idle, Marked, Leaving }

#[derive(Clone)]
struct Block {
    id: u64,
    text: String,
    phase: Phase,
    variant: Variant,
}

fn initial_blocks() -> Vec<Block> {
    INITIAL_BLOCKS.iter().enumerate().map(|(i, t)| Block {
        id: (i as u64) + 1,
        text: (*t).to_string(),
        phase: Phase::Idle,
        variant: Variant::RedFade,
    }).collect()
}

fn prefers_reduced_motion() -> bool {
    let Some(win) = web_sys::window() else { return false; };
    match win.match_media("(prefers-reduced-motion: reduce)") {
        Ok(Some(m)) => m.matches(),
        Ok(None) => false,
        Err(_) => false,
    }
}

#[component]
pub fn DeletingBlock() -> impl IntoView {
    let blocks = RwSignal::new(initial_blocks());
    let active_variant = RwSignal::new(Variant::RedFade);
    let handles: RwSignal<Vec<TimeoutHandle>> = RwSignal::new(Vec::new());

    on_cleanup(move || {
        let list = handles.get_untracked();
        for h in list { h.clear(); }
    });

    let root_style = format!(
        "--db-red-mark: {}ms; --db-red-leave: {}ms; \
         --db-strike-mark: {}ms; --db-strike-leave: {}ms; \
         --db-drop-mark: {}ms; --db-drop-leave: {}ms;",
        RED_MARK_MS, RED_LEAVE_MS,
        STRIKE_MARK_MS, STRIKE_LEAVE_MS,
        DROP_MARK_MS, DROP_LEAVE_MS,
    );

    let schedule_delete = Callback::new(move |id: u64| {
        let reduced = prefers_reduced_motion();
        let v = active_variant.get_untracked();
        let (mark_ms, leave_ms) = if reduced {
            (REDUCED_MARK_MS, REDUCED_LEAVE_MS)
        } else {
            (v.mark_ms(), v.leave_ms())
        };
        blocks.update(|list| {
            if let Some(b) = list.iter_mut().find(|b| b.id == id) {
                if b.phase != Phase::Idle { return; }
                b.phase = Phase::Marked;
                b.variant = v;
            }
        });
        let h_mark = set_timeout_with_handle(
            move || {
                blocks.update(|list| {
                    if let Some(b) = list.iter_mut().find(|b| b.id == id && b.phase == Phase::Marked) {
                        b.phase = Phase::Leaving;
                    }
                });
                let h_leave = set_timeout_with_handle(
                    move || {
                        blocks.update(|list| list.retain(|b| b.id != id));
                    },
                    Duration::from_millis((leave_ms + REMOVE_MARGIN_MS) as u64),
                );
                match h_leave {
                    Ok(h) => handles.update(|v| v.push(h)),
                    Err(e) => {
                        leptos::logging::error!("deleting_block: leave timeout: {e:?}");
                        blocks.update(|list| list.retain(|b| b.id != id));
                    }
                }
            },
            Duration::from_millis(mark_ms as u64),
        );
        match h_mark {
            Ok(h) => handles.update(|v| v.push(h)),
            Err(e) => {
                leptos::logging::error!("deleting_block: mark timeout: {e:?}");
                blocks.update(|list| list.retain(|b| b.id != id));
            }
        }
    });

    let on_reset = move |_| {
        let list = handles.get_untracked();
        for h in list { h.clear(); }
        handles.set(Vec::new());
        blocks.set(initial_blocks());
    };

    view! {
        <div class="db-root" style=root_style>
            <div class="db-toolbar">
                <div class="seg">
                    {[Variant::RedFade, Variant::StrikeSlide, Variant::Drop].iter().map(|&v| {
                        let is_active = move || active_variant.get() == v;
                        view! {
                            <button
                                class="seg-btn"
                                class:active=is_active
                                on:click=move |_| active_variant.set(v)
                            >
                                {v.label()}
                            </button>
                        }
                    }).collect_view()}
                </div>
                <button class="db-btn" on:click=on_reset>"Reset"</button>
            </div>
            <div class="db-list">
                <For
                    each=move || blocks.get()
                    key=|b| b.id
                    children=move |b: Block| {
                        let id = b.id;
                        let text = b.text.clone();
                        let variant_attr = move || {
                            blocks.with(|list| {
                                list.iter()
                                    .find(|x| x.id == id)
                                    .map(|x| x.variant.attr())
                                    .unwrap_or("red")
                            })
                        };
                        let phase_attr = move || {
                            let phase = blocks.with(|list| {
                                list.iter().find(|x| x.id == id).map(|x| x.phase).unwrap_or(Phase::Idle)
                            });
                            match phase {
                                Phase::Idle => "",
                                Phase::Marked => "marked",
                                Phase::Leaving => "leaving",
                            }
                        };
                        view! {
                            <div class="db-row" data-variant=variant_attr data-phase=phase_attr>
                                <div class="db-row-clip">
                                    <div class="db-block">
                                        <span class="db-text">{text}</span>
                                        <button class="db-btn" on:click=move |_| schedule_delete.run(id)>
                                            "Delete"
                                        </button>
                                    </div>
                                </div>
                            </div>
                        }
                    }
                />
            </div>
        </div>
    }
}