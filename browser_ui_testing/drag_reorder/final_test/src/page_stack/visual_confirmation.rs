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
                        <div class="ps-guide-line ps-guide-h" style:top=format!("{}px", r.y) style:background=color />
                        <div class="ps-guide-line ps-guide-h" style:top=format!("{}px", r.y + r.h) style:background=color />
                    </>
                }
            }).collect_view()
        })}
    }
}
