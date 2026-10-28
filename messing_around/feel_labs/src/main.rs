use leptos::prelude::*;

mod block_cascade;
mod block_transition;
mod deleting_block;
mod drag_reorder_lab;
mod idle_page;
mod main_menu;
mod new_page_popup;
mod organic_split;
mod page_entrance;
mod popups;
mod running_into_wall;
mod search_bar;
mod settings_modal;
mod two_version;

use block_cascade::BlockCascade;
use block_transition::BlockTransition;
use deleting_block::DeletingBlock;
use drag_reorder_lab::DragReorderLab;
use idle_page::IdlePage;
use main_menu::MainMenu;
use new_page_popup::NewPagePopup;
use organic_split::OrganicSplit;
use page_entrance::PageEntrance;
use popups::Popups;
use running_into_wall::RunningIntoWall;
use search_bar::SearchBar;
use settings_modal::SettingsModal;
use two_version::TwoVersion;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum LabGroup { Typing, Blocks, PagesAndChrome }

impl LabGroup {
    fn label(self) -> &'static str {
        match self {
            LabGroup::Typing => "Typing",
            LabGroup::Blocks => "Blocks",
            LabGroup::PagesAndChrome => "Pages and chrome",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Lab {
    BlockTransition, RunningIntoWall, SearchBar,
    DeletingBlock, BlockCascade, DragReorder, PageEntrance,
    IdlePage, NewPagePopup, MainMenu, Popups, TwoVersion, SettingsModal, OrganicSplit,
}

impl Lab {
    fn label(self) -> &'static str {
        match self {
            Lab::BlockTransition => "Block transition",
            Lab::RunningIntoWall => "Running into the wall",
            Lab::SearchBar => "Search bar",
            Lab::DeletingBlock => "Deleting a block",
            Lab::BlockCascade => "Block cascade",
            Lab::DragReorder => "Drag reorder",
            Lab::PageEntrance => "Page entrance",
            Lab::IdlePage => "Idle page",
            Lab::NewPagePopup => "New page popup",
            Lab::MainMenu => "Main menu",
            Lab::Popups => "Popups",
            Lab::TwoVersion => "Two-version page",
            Lab::SettingsModal => "Settings modal",
            Lab::OrganicSplit => "Organic split",
        }
    }

    fn group(self) -> LabGroup {
        match self {
            Lab::BlockTransition | Lab::RunningIntoWall | Lab::SearchBar => LabGroup::Typing,
            Lab::DeletingBlock | Lab::BlockCascade | Lab::DragReorder | Lab::PageEntrance => LabGroup::Blocks,
            Lab::IdlePage | Lab::NewPagePopup | Lab::MainMenu | Lab::Popups | Lab::TwoVersion | Lab::SettingsModal | Lab::OrganicSplit => LabGroup::PagesAndChrome,
        }
    }

    fn blurb(self) -> &'static str {
        match self {
            Lab::BlockTransition => "A new line is appended to two preview panes; each pane shows a different easing for how that line arrives.",
            Lab::RunningIntoWall => "Text types itself and three small dots trail across each row that fills.",
            Lab::SearchBar => "The look of typing into a search field with fake results appearing and updating below.",
            Lab::DeletingBlock => "Delete a block. Three exit variants selectable via a segmented control; each locks per block at click time.",
            Lab::BlockCascade => "Delete or insert a block and everything below moves. Three motion variants to compare.",
            Lab::DragReorder => "Physics block reorder from the real app: drag between pages, ghost into the vacuum strip, split on Enter, merge on Backspace.",
            Lab::PageEntrance => "A mock page whose title and blocks enter on mount. Three entrance variants.",
            Lab::IdlePage => "Three takes on what a page does when you leave it alone.",
            Lab::NewPagePopup => "A mock workspace with a new-page dialog over it. Newly typed letters shake.",
            Lab::MainMenu => "A page-list menu with a pen and eye per row. Three hover effects to compare.",
            Lab::Popups => "A toast stack. Click a popup to inspect it as a modal; auto-dismiss pauses while the modal is open.",
            Lab::TwoVersion => "Two drafts side by side. Pick one, edit it, confirm — the losing draft pops out and the winner shines.",
            Lab::SettingsModal => "A designed settings modal. Six categories, a variety of control types, no save.",
            Lab::OrganicSplit => "A block splitting into a new block, in the app's own style.",
        }
    }
}

const LABS: [Lab; 14] = [
    Lab::BlockTransition, Lab::RunningIntoWall, Lab::SearchBar,
    Lab::DeletingBlock, Lab::BlockCascade, Lab::DragReorder, Lab::PageEntrance,
    Lab::IdlePage, Lab::NewPagePopup, Lab::MainMenu,
    Lab::Popups, Lab::TwoVersion, Lab::SettingsModal, Lab::OrganicSplit,
];

#[component]
fn App() -> impl IntoView {
    let selected = RwSignal::new(Lab::BlockTransition);
    let mut grouped: Vec<(LabGroup, Vec<Lab>)> = Vec::new();
    for lab in LABS {
        match grouped.last_mut() {
            Some((g, v)) if *g == lab.group() => v.push(lab),
            _ => grouped.push((lab.group(), vec![lab])),
        }
    }
    let picker = grouped.into_iter().map(|(group, labs)| {
        let buttons = labs.into_iter().map(|lab| {
            let is_active = move || selected.get() == lab;
            view! {
                <button class="picker-button" class:active=is_active on:click=move |_| selected.set(lab)>
                    {lab.label()}
                </button>
            }
        }).collect_view();
        view! {
            <div class="picker-group">
                <span class="picker-group-label">{group.label()}</span>
                <div class="picker-group-buttons">{buttons}</div>
            </div>
        }
    }).collect_view();
    view! {
        <div class="app">
            <header class="shell-header">
                <span class="wordmark">"feel_labs" <span class="wordmark-dot"></span></span>
                <nav class="picker" aria-label="Lab picker">{picker}</nav>
            </header>
            <main class="lab-area">
                <div class="lab-head">
                    <h1 class="lab-title">{move || selected.get().label()}</h1>
                    <p class="lab-blurb">{move || selected.get().blurb()}</p>
                </div>
                <div class="lab-stage">
                    {move || match selected.get() {
                        Lab::BlockTransition => view! { <BlockTransition /> }.into_any(),
                        Lab::RunningIntoWall => view! { <RunningIntoWall /> }.into_any(),
                        Lab::SearchBar => view! { <SearchBar /> }.into_any(),
                        Lab::DeletingBlock => view! { <DeletingBlock /> }.into_any(),
                        Lab::BlockCascade => view! { <BlockCascade /> }.into_any(),
                        Lab::DragReorder => view! { <DragReorderLab /> }.into_any(),
                        Lab::PageEntrance => view! { <PageEntrance /> }.into_any(),
                        Lab::IdlePage => view! { <IdlePage /> }.into_any(),
                        Lab::NewPagePopup => view! { <NewPagePopup /> }.into_any(),
                        Lab::MainMenu => view! { <MainMenu /> }.into_any(),
                        Lab::Popups => view! { <Popups /> }.into_any(),
                        Lab::TwoVersion => view! { <TwoVersion /> }.into_any(),
                        Lab::SettingsModal => view! { <SettingsModal /> }.into_any(),
                        Lab::OrganicSplit => view! { <OrganicSplit /> }.into_any(),
                    }}
                </div>
            </main>
        </div>
    }
}