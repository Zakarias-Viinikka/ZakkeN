use block_diff_viewer_leptos::{DiffViewer, FinishHandle};
use leptos::prelude::*;

use crate::sample_pages::{sample_pairs, SamplePair};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    PickerAndPreview,
    DiffViewer,
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
            <header class="app-pair-menu">
                {buttons}
            </header>
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
    let left_original = left.clone();
    let right_original = right.clone();
    let left_signal = RwSignal::new(left);
    let right_signal = RwSignal::new(right);
    let (output, set_output) = signal(Vec::<String>::new());
    let handle = FinishHandle::new();
    let phase = RwSignal::new(Phase::Editing);

    let on_finished = Callback::new(move |_: ()| {
        phase.set(Phase::ShowingResult);
    });

    let reopen = Callback::new(move |_: ()| {
        left_signal.set(left_original.clone());
        right_signal.set(right_original.clone());
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
                        let blocks = output.get();
                        view! {
                            <div class="app-result">
                                <div class="app-result-header">
                                    <span>{format!("Result: {} blocks", blocks.len())}</span>
                                    <button on:click=move |_| reopen.run(())>"Open again"</button>
                                </div>
                                <ol class="app-result-list">
                                    {blocks
                                        .into_iter()
                                        .map(|text| view! { <li class="app-result-card">{text}</li> })
                                        .collect_view()}
                                </ol>
                            </div>
                        }
                    }
                >
                    <DiffViewer
                        left_page=left_signal
                        right_page=right_signal
                        output=set_output
                        handle=handle
                        on_finished=on_finished
                    />
                </Show>
            </div>
        </div>
    }
}