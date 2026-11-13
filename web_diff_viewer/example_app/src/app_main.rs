use block_diff_viewer_leptos::{
    Block, DiffViewer, FinishHandle, PageBlocksForTheViewer, PickedSide, ViewerOutput,
};
use leptos::prelude::*;

use crate::sample_pages::{sample_pairs, SamplePair};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    PickerAndPreview,
    DiffViewer,
}

fn page_from_texts(page_id: &str, texts: Vec<String>) -> PageBlocksForTheViewer {
    PageBlocksForTheViewer {
        page_id: page_id.to_string(),
        is_main_menu_page: false,
        blocks: texts
            .into_iter()
            .enumerate()
            .map(|(i, text)| Block {
                block_id: format!("{page_id}-{i}"),
                text,
                metadata: String::new(),
                frac_position: format!("{}", i),
            })
            .collect(),
    }
}

#[component]
pub fn AppMain() -> impl IntoView {
    let selected: RwSignal<usize> = RwSignal::new(0);
    let screen: RwSignal<Screen> = RwSignal::new(Screen::PickerAndPreview);

    view! {
        {move || {
            let index = selected.get();
            let pairs = sample_pairs();
            let pair = pairs
                .into_iter()
                .nth(index)
                .unwrap_or_else(|| sample_pairs().into_iter().next().unwrap());
            match screen.get() {
                Screen::PickerAndPreview => view! {
                    <PickerAndPreview
                        selected=index
                        on_select=Callback::new(move |i: usize| selected.set(i))
                        pair_name=pair.name
                        left=pair.left
                        right=pair.right
                        on_open_diff=Callback::new(move |_: ()| {
                            screen.set(Screen::DiffViewer);
                        })
                    />
                }
                .into_any(),
                Screen::DiffViewer => view! {
                    <SampleViewer
                        pair=pair
                        on_back=Callback::new(move |_: ()| {
                            screen.set(Screen::PickerAndPreview);
                        })
                    />
                }
                .into_any(),
            }
        }}
    }
}

#[component]
fn PickerAndPreview(
    selected: usize,
    on_select: Callback<usize>,
    pair_name: &'static str,
    left: Vec<String>,
    right: Vec<String>,
    on_open_diff: Callback<()>,
) -> impl IntoView {
    let pairs = sample_pairs();
    let left_blocks = left;
    let right_blocks = right;
    let buttons = pairs
        .into_iter()
        .enumerate()
        .map(|(i, pair)| {
            let name = pair.name;
            let is_selected = i == selected;
            view! {
                <button
                    type="button"
                    class=if is_selected {
                        "app-pair-button app-pair-button-active"
                    } else {
                        "app-pair-button"
                    }
                    on:click=move |_| on_select.run(i)
                >
                    {name}
                </button>
            }
        })
        .collect_view();

    view! {
        <div class="app-preview-page">
            <header class="app-pair-menu">{buttons}</header>
            <div class="app-preview-panes">
                <div class="app-preview-pane">
                    <div class="app-preview-pane-title">"Before"</div>
                    <div class="app-preview-pane-body">
                        {left_blocks
                            .into_iter()
                            .map(|text| view! { <div class="app-preview-block">{text}</div> })
                            .collect_view()}
                    </div>
                </div>
                <div class="app-preview-pane">
                    <div class="app-preview-pane-title">"After"</div>
                    <div class="app-preview-pane-body">
                        {right_blocks
                            .into_iter()
                            .map(|text| view! { <div class="app-preview-block">{text}</div> })
                            .collect_view()}
                    </div>
                </div>
            </div>
            <footer class="app-preview-footer">
                <span class="app-preview-selected">"Selected: " {pair_name}</span>
                <button on:click=move |_| on_open_diff.run(())>
                    "View diff viewer →"
                </button>
            </footer>
        </div>
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Editing,
    ShowingResult,
}

#[component]
fn SampleViewer(pair: SamplePair, on_back: Callback<()>) -> impl IntoView {
    let SamplePair { name, description: _, left, right } = pair;
    let page_id = format!("sample-{name}");
    let left_original = page_from_texts(&page_id, left);
    let right_original = page_from_texts(&page_id, right);

    let server_signal = RwSignal::new(Some(left_original.clone()));
    let client_signal = RwSignal::new(right_original.clone());
    let kept_side_signal = RwSignal::new(PickedSide::Mine);
    let (output, set_output) = signal(ViewerOutput::default());
    let handle = FinishHandle::new();
    let phase = RwSignal::new(Phase::Editing);

    let on_finished = Callback::new(move |_: ()| {
        phase.set(Phase::ShowingResult);
    });

    let reopen = Callback::new(move |_: ()| {
        server_signal.set(Some(left_original.clone()));
        client_signal.set(right_original.clone());
        phase.set(Phase::Editing);
    });

    view! {
        <div class="app-root">
            <header class="app-toolbar">
                <button on:click=move |_| on_back.run(())>"← Back"</button>
                <h1>{name}</h1>
                <button on:click=move |_| handle.finish_and_write_the_result()>
                    "Done"
                </button>
            </header>
            <div class="app-viewer-host">
                <Show
                    when=move || phase.get() == Phase::Editing
                    fallback=move || {
                        let kept = output.get().kept_blocks;
                        view! {
                            <div class="app-result">
                                <div class="app-result-header">
                                    <span>{format!("Result: {} blocks", kept.len())}</span>
                                    <button on:click=move |_| reopen.run(())>"Open again"</button>
                                </div>
                                <ol class="app-result-list">
                                    {kept
                                        .into_iter()
                                        .map(|b| view! { <li class="app-result-card">{b.text}</li> })
                                        .collect_view()}
                                </ol>
                            </div>
                        }
                    }
                >
                    <DiffViewer
                        server_page=server_signal
                        client_page=client_signal
                        kept_side=kept_side_signal
                        output=set_output
                        handle=handle
                        on_finished=on_finished
                    />
                </Show>
            </div>
        </div>
    }
}