use leptos::logging::error;
use leptos::prelude::*;

const SIZE: f64 = 4000.0;
const GAP: f64 = 30.0;

const BUTTONS: [(f64, f64, &str); 5] = [
    (50.0, 50.0, "Center"),
    (25.0, 25.0, "Top Left"),
    (75.0, 25.0, "Top Right"),
    (25.0, 75.0, "Bottom Left"),
    (75.0, 75.0, "Bottom Right"),
];

#[component]
pub fn Theme7() -> impl IntoView {
    let cx = RwSignal::new(50.0_f64);
    let cy = RwSignal::new(50.0_f64);
    let delay1 = RwSignal::new(0_u32);
    let delay2 = RwSignal::new(0_u32);

    let offset = (SIZE + GAP) / (2.0 * std::f64::consts::SQRT_2);

    let on_click = move |target_cx: f64, target_cy: f64| {
        let (vw, vh) = match window_size() {
            Ok(v) => v,
            Err(e) => {
                error!("{e}");
                return;
            }
        };

        let cur_cx = cx.get_untracked();
        let cur_cy = cy.get_untracked();

        let box1_x = cur_cx / 100.0 * vw - offset;
        let box1_y = cur_cy / 100.0 * vh + offset;
        let box2_x = cur_cx / 100.0 * vw + offset;
        let box2_y = cur_cy / 100.0 * vh - offset;

        let target_x = target_cx / 100.0 * vw;
        let target_y = target_cy / 100.0 * vh;

        let dist1 = ((box1_x - target_x).powi(2) + (box1_y - target_y).powi(2)).sqrt();
        let dist2 = ((box2_x - target_x).powi(2) + (box2_y - target_y).powi(2)).sqrt();

        delay1.set(if dist1 <= dist2 { 0 } else { 200 });
        delay2.set(if dist2 < dist1 { 0 } else { 200 });

        cx.set(target_cx);
        cy.set(target_cy);
    };

    let box1_style = move || {
        format!(
            "left: calc({}% - {}px); top: calc({}% + {}px); transition-delay: {}ms;",
            cx.get(),
            offset,
            cy.get(),
            offset,
            delay1.get(),
        )
    };
    let box2_style = move || {
        format!(
            "left: calc({}% + {}px); top: calc({}% - {}px); transition-delay: {}ms;",
            cx.get(),
            offset,
            cy.get(),
            offset,
            delay2.get(),
        )
    };

    view! {
        <div class="box box1" style=box1_style />
        <div class="box box2" style=box2_style />

        <nav class="nav-container">
            {BUTTONS
                .into_iter()
                .map(|(bx, by, label)| {
                    view! {
                        <button class="nav-btn" on:click=move |_| on_click(bx, by)>
                            {label}
                        </button>
                    }
                })
                .collect_view()}
        </nav>
    }
}

fn window_size() -> Result<(f64, f64), String> {
    let win = web_sys::window().ok_or_else(|| "no window".to_string())?;
    let w = win
        .inner_width()
        .map_err(|e| format!("inner_width: {:?}", e))?
        .as_f64()
        .ok_or_else(|| "inner_width not a number".to_string())?;
    let h = win
        .inner_height()
        .map_err(|e| format!("inner_height: {:?}", e))?
        .as_f64()
        .ok_or_else(|| "inner_height not a number".to_string())?;
    Ok((w, h))
}
