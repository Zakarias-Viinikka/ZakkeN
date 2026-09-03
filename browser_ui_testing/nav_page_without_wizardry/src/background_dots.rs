use js_sys::Math;
use leptos::prelude::*;
use leptos_meta::Stylesheet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

static DOT_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug)]
struct DotSpec {
    id: u64,
    left: String,
    top: String,
    width: String,
    height: String,
    tx: String,
    ty: String,
    duration: f64, // seconds of movement
    direction: String,
    fading: bool, // whether the fade‑out class should be applied
}

#[component]
pub fn BackgroundDots() -> impl IntoView {
    let (dots, set_dots) = signal(Vec::<DotSpec>::new());

    // Spawn a dot every 800 ms with 40% chance
    set_interval(
        move || {
            if Math::random() < 0.4 {
                let dot = generate_dot();
                let id = dot.id;
                let duration = dot.duration;

                // Add the new dot to the list
                set_dots.update(|dots_vec| {
                    // Keep the list bounded (optional)
                    if dots_vec.len() >= 50 {
                        dots_vec.remove(0);
                    }
                    dots_vec.push(dot);
                });

                // After its movement duration, start the fade‑out
                set_timeout(
                    move || {
                        // Set fading = true for the dot with this id
                        set_dots.update(|dots_vec| {
                            if let Some(dot) = dots_vec.iter_mut().find(|d| d.id == id) {
                                dot.fading = true;
                            }
                        });

                        // After the fade‑out animation finishes, delete the dot
                        set_timeout(
                            move || {
                                set_dots.update(|dots_vec| {
                                    if let Some(pos) = dots_vec.iter().position(|d| d.id == id) {
                                        dots_vec.remove(pos);
                                    }
                                });
                            },
                            Duration::from_millis(500), // match the CSS fade‑out duration
                        );
                    },
                    Duration::from_millis((duration * 1000.0) as u64),
                );
            }
        },
        Duration::from_millis(800),
    );

    view! {
        <Stylesheet href="/css/background_dots.css" />
        <div class="background-dots" aria-hidden="true">
            <For
                each=move || dots.get()
                key=|dot| dot.id
                let(dot)
            >
                <div
                    class="dot"
                    class:fade-out=dot.fading
                    style=format!(
                        "left: {}; top: {}; width: {}; height: {}; \
                         animation: floatRandom {:.1}s ease-in-out 1 {} forwards; \
                         --tx: {}; --ty: {};",
                        dot.left, dot.top, dot.width, dot.height,
                        dot.duration, dot.direction, dot.tx, dot.ty
                    )
                ></div>
            </For>
        </div>
    }
}

fn generate_dot() -> DotSpec {
    let id = DOT_ID.fetch_add(1, Ordering::Relaxed);
    let left = format!("{:.1}%", Math::random() * 100.0);
    let top = format!("{:.1}%", Math::random() * 100.0);

    // Size between 2px and 5px
    let size = 2.0 + Math::random() * 3.0;

    // Random translation target: -150px to 150px
    let tx = format!("{:.0}px", (Math::random() - 0.5) * 300.0);
    let ty = format!("{:.0}px", (Math::random() - 0.5) * 300.0);

    // Movement duration between 8s and 20s
    let duration = 8.0 + Math::random() * 12.0;

    let direction = if Math::random() < 0.5 {
        "normal"
    } else {
        "reverse"
    };

    DotSpec {
        id,
        left,
        top,
        width: format!("{:.1}px", size),
        height: format!("{:.1}px", size),
        tx,
        ty,
        duration,
        direction: direction.to_string(),
        fading: false,
    }
}
