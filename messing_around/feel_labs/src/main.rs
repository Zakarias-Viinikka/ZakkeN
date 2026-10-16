use leptos::prelude::*;

mod block_transition;
mod deleting_block;
mod idle_page;
mod new_page_popup;
mod organic_split;
mod running_into_wall;

use block_transition::BlockTransition;
use deleting_block::DeletingBlock;
use idle_page::IdlePage;
use new_page_popup::NewPagePopup;
use organic_split::OrganicSplit;
use running_into_wall::RunningIntoWall;

fn main() {
    console_error_panic_hook::set_once();
    //  trunk serve --open

    mount_to_body(App);
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Lab {
    BlockTransition,
    RunningIntoWall,
    DeletingBlock,
    IdlePage,
    NewPagePopup,
    OrganicSplit,
}

impl Lab {
    fn label(self) -> &'static str {
        match self {
            Lab::BlockTransition => "Block transition",
            Lab::RunningIntoWall => "Running into the wall",
            Lab::DeletingBlock => "Deleting a block",
            Lab::IdlePage => "Idle page",
            Lab::NewPagePopup => "New page popup",
            Lab::OrganicSplit => "Organic split",
        }
    }
}

const LABS: [Lab; 6] = [
    Lab::BlockTransition,
    Lab::RunningIntoWall,
    Lab::DeletingBlock,
    Lab::IdlePage,
    Lab::NewPagePopup,
    Lab::OrganicSplit,
];

#[component]
fn App() -> impl IntoView {
    let selected = RwSignal::new(Lab::BlockTransition);

    let picker = LABS
        .into_iter()
        .map(|lab| {
            let is_active = move || selected.get() == lab;
            view! {
                <button
                    class="picker-button"
                    class:active=is_active
                    on:click=move |_| selected.set(lab)
                >
                    {lab.label()}
                </button>
            }
        })
        .collect_view();

    view! {
        <div class="app">
            <nav class="picker" aria-label="Lab picker">
                {picker}
            </nav>
            <main class="lab-area">
                {move || match selected.get() {
                    Lab::BlockTransition => view! { <BlockTransition /> }.into_any(),
                    Lab::RunningIntoWall => view! { <RunningIntoWall /> }.into_any(),
                    Lab::DeletingBlock => view! { <DeletingBlock /> }.into_any(),
                    Lab::IdlePage => view! { <IdlePage /> }.into_any(),
                    Lab::NewPagePopup => view! { <NewPagePopup /> }.into_any(),
                    Lab::OrganicSplit => view! { <OrganicSplit /> }.into_any(),
                }}
            </main>
        </div>
    }
}