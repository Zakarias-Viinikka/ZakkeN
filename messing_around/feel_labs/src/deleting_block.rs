use leptos::prelude::*;
use std::time::Duration;
use stylist::style;

const RED_HOLD_MS: u32 = 400;
const EXIT_MS: u32 = 320;
const POP_SCALE: f64 = 0.85;
const POP_DISTANCE_PX: i32 = 8;

const RED_HOLD_REDUCED_MS: u32 = 120;
const EXIT_REDUCED_MS: u32 = 1;

const INITIAL_BLOCKS: &[&str] = &[
    "First block of text.",
    "Second block of text.",
    "Third block of text.",
    "Fourth block of text.",
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Idle,
    Red,
    Exit,
}

#[derive(Clone, PartialEq)]
struct Block {
    id: u64,
    text: String,
    phase: Phase,
}

fn initial_blocks() -> Vec<Block> {
    INITIAL_BLOCKS
        .iter()
        .enumerate()
        .map(|(i, t)| Block {
            id: (i as u64) + 1,
            text: (*t).to_string(),
            phase: Phase::Idle,
        })
        .collect()
}

fn prefers_reduced_motion() -> bool {
    let Some(win) = web_sys::window() else {
        return false;
    };
    match win.match_media("(prefers-reduced-motion: reduce)") {
        Ok(Some(m)) => m.matches(),
        Ok(None) => false,
        Err(e) => {
            leptos::logging::error!("deleting_block: match_media: {e:?}");
            false
        }
    }
}

#[component]
pub fn DeletingBlock() -> AnyView {
    let styles = match style!(
        r#"
.db-root {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 16px;
    --db-pop-scale: ${pop_scale};
    --db-pop-dist: ${pop_dist}px;
    --db-exit-dur: ${exit_ms}ms;
}

.db-toolbar {
    display: flex;
    gap: 8px;
}

.db-btn {
    font: inherit;
    padding: 6px 12px;
    border: 1px solid rgba(128, 128, 128, 0.5);
    border-radius: 6px;
    background: transparent;
    color: inherit;
    cursor: pointer;
}

.db-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
}

.db-block {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px 12px;
    border: 1px solid rgba(128, 128, 128, 0.4);
    border-radius: 6px;
    font-family: system-ui, -apple-system, "Segoe UI", Roboto, sans-serif;
    font-size: 15px;
    transition: color 220ms ease;
}

.db-red {
    color: #d64545;
}

.db-exit {
    animation: db-pop var(--db-exit-dur, 320ms) ease-in forwards;
}

@keyframes db-pop {
    from { transform: translateY(0) scale(1); opacity: 1; }
    to   { transform: translateY(var(--db-pop-dist, 8px)) scale(var(--db-pop-scale, 0.85)); opacity: 0; }
}

@media (prefers-reduced-motion: reduce) {
    .db-exit { animation-duration: 1ms; }
}
"#,
        pop_scale = POP_SCALE,
        pop_dist = POP_DISTANCE_PX,
        exit_ms = EXIT_MS,
    ) {
        Ok(s) => s,
        Err(e) => {
            leptos::logging::error!("deleting_block: style build failed: {e}");
            return view! {
                <div>"Deleting a block styles failed to build. See console."</div>
            }
            .into_any();
        }
    };
    let root_class = styles.get_class_name().to_string();

    let blocks = RwSignal::new(initial_blocks());
    let next_id = RwSignal::new(INITIAL_BLOCKS.len() as u64 + 1);
    let handles = RwSignal::new(Vec::<TimeoutHandle>::new());

    on_cleanup(move || {
        let list = handles.get_untracked();
        for h in list {
            h.clear();
        }
    });

    let schedule_delete = move |id: u64| {
        let reduced = prefers_reduced_motion();
        let (red_ms, exit_ms) = if reduced {
            (RED_HOLD_REDUCED_MS, EXIT_REDUCED_MS)
        } else {
            (RED_HOLD_MS, EXIT_MS)
        };

        blocks.update(|list| {
            if let Some(b) = list.iter_mut().find(|b| b.id == id) {
                b.phase = Phase::Red;
            }
        });

        let h_red = set_timeout_with_handle(
            move || {
                blocks.update(|list| {
                    if let Some(b) = list.iter_mut().find(|b| b.id == id) {
                        b.phase = Phase::Exit;
                    }
                });
                let h_exit = set_timeout_with_handle(
                    move || {
                        blocks.update(|list| list.retain(|b| b.id != id));
                    },
                    Duration::from_millis(exit_ms as u64),
                );
                match h_exit {
                    Ok(h) => handles.update(|v| v.push(h)),
                    Err(e) => {
                        leptos::logging::error!(
                            "deleting_block: schedule exit timeout: {e:?}"
                        );
                        blocks.update(|list| list.retain(|b| b.id != id));
                    }
                }
            },
            Duration::from_millis(red_ms as u64),
        );
        match h_red {
            Ok(h) => handles.update(|v| v.push(h)),
            Err(e) => {
                leptos::logging::error!("deleting_block: schedule red timeout: {e:?}");
                blocks.update(|list| list.retain(|b| b.id != id));
            }
        }
    };

    let on_reset = move |_| {
        let list = handles.get_untracked();
        for h in list {
            h.clear();
        }
        handles.set(Vec::new());
        next_id.set(INITIAL_BLOCKS.len() as u64 + 1);
        blocks.set(initial_blocks());
    };

    view! {
        <div class=root_class>
            <div class="db-toolbar">
                <button class="db-btn" on:click=on_reset>"Reset"</button>
            </div>
            <div class="db-list">
                <For
                    each=move || blocks.get()
                    key=|b| b.id
                    children=move |b: Block| {
                        let id = b.id;
                        let text = b.text.clone();
                        let class_closure = move || {
                            let phase = blocks.with(|list| {
                                list.iter()
                                    .find(|x| x.id == id)
                                    .map(|x| x.phase)
                                    .unwrap_or(Phase::Idle)
                            });
                            match phase {
                                Phase::Idle => "db-block".to_string(),
                                Phase::Red => "db-block db-red".to_string(),
                                Phase::Exit => "db-block db-exit".to_string(),
                            }
                        };
                        view! {
                            <div class=class_closure>
                                <span>{text}</span>
                                <button
                                    class="db-btn"
                                    on:click=move |_| schedule_delete(id)
                                >
                                    "Delete"
                                </button>
                            </div>
                        }
                    }
                />
            </div>
        </div>
    }
    .into_any()
}