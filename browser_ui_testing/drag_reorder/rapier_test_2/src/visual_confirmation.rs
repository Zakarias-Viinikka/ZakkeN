use leptos::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Clone, Debug)]
pub enum Confirmation {
    One(Rect),
    Many(Vec<Rect>),
}

const PALETTE: [&str; 4] = ["#8ab4ff", "#f5a97f", "#a6da95", "#eed49f"];

/// Draws four full-page guide lines per rect — top edge, bottom edge, left
/// edge, right edge. `None` renders nothing. A new value replaces the old
/// wholesale. Each rect gets its own colour from the palette.
#[component]
pub fn FourLines(confirmation: RwSignal<Option<Confirmation>>) -> impl IntoView {
    view! {
        {move || confirmation.get().map(|c| {
            let rects: Vec<Rect> = match c {
                Confirmation::One(r) => vec![r],
                Confirmation::Many(v) => v,
            };
            rects.into_iter().enumerate().map(|(i, r)| {
                let color = PALETTE[i % PALETTE.len()];
                view! {
                    <>
                        <div class="guide-line guide-h" style:top=format!("{}px", r.y) style:background=color />
                        <div class="guide-line guide-h" style:top=format!("{}px", r.y + r.h) style:background=color />
                        <div class="guide-line guide-v" style:left=format!("{}px", r.x) style:background=color />
                        <div class="guide-line guide-v" style:left=format!("{}px", r.x + r.w) style:background=color />
                    </>
                }
            }).collect_view()
        })}
    }
}