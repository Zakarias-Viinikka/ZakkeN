use leptos::prelude::*;

const BLUR_STD_DEV: f64 = 16.0;
const ALPHA_SLOPE: f64 = 12.0;
const ALPHA_INTERCEPT: f64 = -6.0;

const VIEWBOX_W: f64 = 400.0;
const VIEWBOX_H: f64 = 500.0;

const SQUARE_SIZE: f64 = 150.0;
const SQUARE_RX: f64 = 34.0;
const SQUARE_X: f64 = 125.0;
const SQUARE_Y: f64 = 60.0;

const FILTER_ID: &str = "feel-labs-organic-split-metaball";

#[component]
fn OrganicSvg(splitting: bool) -> impl IntoView {
    let class = if splitting { "os-svg os-splitting" } else { "os-svg" };
    let viewbox = format!("0 0 {} {}", VIEWBOX_W, VIEWBOX_H);
    let filter_values = format!(
        "1 0 0 0 0  0 1 0 0 0  0 0 1 0 0  0 0 0 {} {}",
        ALPHA_SLOPE, ALPHA_INTERCEPT
    );
    let filter_ref = format!("url(#{})", FILTER_ID);
    let blur_std = BLUR_STD_DEV.to_string();

    let x_a = SQUARE_X.to_string();
    let y_a = SQUARE_Y.to_string();
    let w_a = SQUARE_SIZE.to_string();
    let h_a = SQUARE_SIZE.to_string();
    let rx_a1 = SQUARE_RX.to_string();
    let rx_a2 = SQUARE_RX.to_string();

    let x_b = SQUARE_X.to_string();
    let y_b = SQUARE_Y.to_string();
    let w_b = SQUARE_SIZE.to_string();
    let h_b = SQUARE_SIZE.to_string();
    let rx_b1 = SQUARE_RX.to_string();
    let rx_b2 = SQUARE_RX.to_string();

    let top_label_center = (SQUARE_Y + SQUARE_SIZE / 2.0).to_string();
    let top_label_top = (SQUARE_Y + SQUARE_SIZE / 2.0).to_string();
    let bottom_label_y = (SQUARE_Y + 220.0 + SQUARE_SIZE / 2.0).to_string();

    view! {
        <svg class=class viewBox=viewbox xmlns="http://www.w3.org/2000/svg" preserveAspectRatio="xMidYMid meet">
            <defs>
                <filter
                    id=FILTER_ID
                    filterUnits="userSpaceOnUse"
                    x="-100"
                    y="-100"
                    width="600"
                    height="800"
                    color-interpolation-filters="sRGB"
                >
                    <feGaussianBlur in="SourceGraphic" stdDeviation=blur_std result="blur" />
                    <feColorMatrix in="blur" type="matrix" values=filter_values />
                </filter>
            </defs>
            <g filter=filter_ref>
                <g class="os-lobe-a">
                    <rect
                        x=x_a
                        y=y_a
                        width=w_a
                        height=h_a
                        rx=rx_a1
                        ry=rx_a2
                        fill="currentColor"
                    />
                </g>
                <g class="os-lobe-b">
                    <rect
                        x=x_b
                        y=y_b
                        width=w_b
                        height=h_b
                        rx=rx_b1
                        ry=rx_b2
                        fill="currentColor"
                    />
                </g>
            </g>
            <text class="os-label os-label-center" x="200" y=top_label_center>"Block"</text>
            <text class="os-label os-label-top" x="200" y=top_label_top>"Top"</text>
            <text class="os-label os-label-bottom" x="200" y=bottom_label_y>"Bottom"</text>
        </svg>
    }
}

#[component]
pub fn OrganicSplit() -> impl IntoView {
    let generation = RwSignal::new(0u64);

    let trigger_split = move |_| {
        generation.update(|g| *g = g.wrapping_add(1));
    };
    let reset = move |_| {
        generation.set(0);
    };

    view! {
        <div class="os-root">
            <div class="os-toolbar">
                <button class="os-btn" on:click=trigger_split>"Split block"</button>
                <button class="os-btn" on:click=reset>"Reset"</button>
            </div>
            <div class="os-stage">
                <For
                    each=move || {
                        let g = generation.get();
                        if g == 0 { vec![0u64] } else { vec![g] }
                    }
                    key=|g| *g
                    children=move |g| {
                        let splitting = g > 0;
                        view! { <OrganicSvg splitting=splitting /> }
                    }
                />
            </div>
        </div>
    }
}