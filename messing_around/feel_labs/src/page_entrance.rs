use leptos::prelude::*;

const BLOCKS: &[&str] = &[
    "A short one.",
    "Second block, mixed length so heights vary.",
    "Third block with a bit more text so the stagger is easier to see.",
    "Fourth.",
    "Fifth block, medium.",
    "Sixth and last.",
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Variant {
    Rise,
    Drop,
    Unfold,
}

impl Variant {
    fn class(self) -> &'static str {
        match self {
            Variant::Rise => "pe-stage pe-v-rise",
            Variant::Drop => "pe-stage pe-v-drop",
            Variant::Unfold => "pe-stage pe-v-unfold",
        }
    }
}

#[component]
pub fn PageEntrance() -> impl IntoView {
    let variant = RwSignal::new(Variant::Rise);
    let generation = RwSignal::new(1u64);

    let replay = move |_| generation.update(|g| *g = g.wrapping_add(1));
    let on_rise = move |_| {
        variant.set(Variant::Rise);
        generation.update(|g| *g = g.wrapping_add(1));
    };
    let on_drop = move |_| {
        variant.set(Variant::Drop);
        generation.update(|g| *g = g.wrapping_add(1));
    };
    let on_unfold = move |_| {
        variant.set(Variant::Unfold);
        generation.update(|g| *g = g.wrapping_add(1));
    };

    let is_rise = move || variant.get() == Variant::Rise;
    let is_drop = move || variant.get() == Variant::Drop;
    let is_unfold = move || variant.get() == Variant::Unfold;

    view! {
        <div class="pe-root">
            <div class="pe-toolbar">
                <div class="seg">
                    <button class="seg-btn" class:active=is_rise on:click=on_rise>"Rise"</button>
                    <button class="seg-btn" class:active=is_drop on:click=on_drop>"Drop"</button>
                    <button class="seg-btn" class:active=is_unfold on:click=on_unfold>"Unfold"</button>
                </div>
                <button class="btn" on:click=replay>"Replay"</button>
            </div>
            <For
                each=move || vec![generation.get()]
                key=|g| *g
                children=move |_g| {
                    let v = variant.get_untracked();
                    view! { <EntrancePage variant=v /> }
                }
            />
        </div>
    }
}

#[component]
fn EntrancePage(variant: Variant) -> impl IntoView {
    let class = variant.class();
    view! {
        <div class=class>
            <div class="mock-page pe-page">
                <div class="pe-title" style="--i: 0">"Field notes"</div>
                {BLOCKS.iter().enumerate().map(|(i, text)| {
                    let style = format!("--i: {}", i + 1);
                    view! { <div class="pe-block mock-block" style=style>{*text}</div> }
                }).collect_view()}
            </div>
        </div>
    }
}