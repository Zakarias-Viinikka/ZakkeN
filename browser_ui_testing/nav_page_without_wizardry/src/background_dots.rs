use js_sys::Math;
use leptos::prelude::*;
use leptos_meta::Stylesheet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

// Global counter for unique dot IDs
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
    duration: f64, // seconds, stored as number for timeout calculation
    direction: String,
}

#[component]
pub fn BackgroundDots() -> impl IntoView {
    let (dots, set_dots) = signal(Vec::<DotSpec>::new());

    set_interval(
        move || {
            if Math::random() < 0.4 {
                let dot = generate_dot();
                let duration = dot.duration;
                let id = dot.id;

                // Add dot to list
                set_dots.update(|dots_vec| {
                    if dots_vec.len() >= 50 {
                        dots_vec.remove(0);
                    }
                    dots_vec.push(dot);
                });

                // Schedule removal after animation completes
                set_timeout(
                    move || {
                        set_dots.update(|dots_vec| {
                            if let Some(pos) = dots_vec.iter().position(|d| d.id == id) {
                                dots_vec.remove(pos);
                            }
                        });
                    },
                    Duration::from_millis((duration * 1000.0) as u64 + 100), // add small buffer
                );
            }
        },
        Duration::from_millis(800),
    );

    view! {
        <Stylesheet href="/css/background_dots.css" />
        <div class="background-dots" aria-hidden="true">
            {move || dots.get().iter().map(|dot| {
                // Animation runs once and ends at opacity 0 (see CSS)
                let style = format!(
                    "left: {}; top: {}; width: {}; height: {}; \
                     animation: floatRandom {:.1}s linear 1 {} forwards; \
                     --tx: {}; --ty: {};",
                    dot.left, dot.top, dot.width, dot.height,
                    dot.duration, dot.direction, dot.tx, dot.ty
                );
                view! {
                    <div class="dot" style={style}></div>
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}

fn generate_dot() -> DotSpec {
    let id = DOT_ID.fetch_add(1, Ordering::Relaxed);
    let left = format!("{:.1}%", Math::random() * 100.0);
    let top = format!("{:.1}%", Math::random() * 100.0);

    // Size between 2px and 5px (smaller in general)
    let size = 2.0 + Math::random() * 3.0;

    // Random translation target: -150px to 150px in both axes
    let tx = format!("{:.0}px", (Math::random() - 0.5) * 300.0);
    let ty = format!("{:.0}px", (Math::random() - 0.5) * 300.0);

    // Duration between 8s and 20s
    let duration = 8.0 + Math::random() * 12.0;

    // Randomly choose normal or reverse animation direction
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
    }
}
