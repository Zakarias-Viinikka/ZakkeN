use crate::app::add_css_and_js;
use crate::app::helper;
use crate::app::js_value_parsing;
use leptos::logging::log;
use leptos::prelude::*;
use leptos::task::spawn_local;
use text_diff2::custom_text_area::CustomTextArea;

use leptos_meta::*;

use js_sys::Array;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

#[derive(Copy, Clone)]
struct TextBlocks {
    text: (ReadSignal<String>, WriteSignal<String>),
    id: usize,
}

impl TextBlocks {
    fn new(id: usize) -> Self {
        Self {
            text: signal(String::new()),
            id,
        }
    }
}

#[component]
pub fn TextBlocksPage() -> impl IntoView {
    provide_meta_context();

    let (list, set_list) = signal(Vec::new());

    set_list.update(|l| {
        for _ in 0..5 {
            l.push(TextBlocks::new(l.len()));
        }
    });

    crate::javascript_take_the_wheel!("update_list_order", |js_value| {
        match js_value_parsing::js_value_to_usize_tuple(js_value) {
            Ok((old_index, new_index)) => {
                set_list.update(|v| {
                    let item = v.remove(old_index);
                    v.insert(new_index, item);
                });
            }
            Err(e) => log!("{}", e),
        }
    });

    Effect::new(move || {
        spawn_local(async move {
            if let Err(e) = helper::create_table_if_not_exist().await {
                log!("create_table_if_not_exist failed: {:?}", e);
            }
        });
    });

    let (last_diff, set_last_diff) = signal(String::new());

    Effect::new(move |_| {
        let diff = last_diff.get();

        if !diff.is_empty() {
            log!("Parent received diff: {}", diff);

            // Later:
            // apply_to_yjs(&diff);
            // send_to_server(&diff);
            // update_database(&diff);
        }
    });

    view! {
        <add_css_and_js::CssAndJs />

        <div class="finale-container">
            <ul id="sortable-container">
                <ForEnumerate
                    each=move || list.get()
                    key=|text_blocks| text_blocks.id
                    let(index, text_blocks)
                >
                    <TextArea
                        index=index
                        on_diff=move |diff: String| {
                            set_last_diff.set(diff);
                        }
                    />
                </ForEnumerate>
            </ul>
        </div>
    }
}

#[component]
fn TextArea(index: ReadSignal<usize>, #[prop(into)] on_diff: Callback<String>) -> impl IntoView {
    view! {
        <li class="text-container" data-id={move || index.get()}>
            <div class="drag-handle">"⠿"</div>

            <div class="text-input-container">
                <CustomTextArea
                    box_index=index.get_untracked()
                    on_diff=on_diff
                    attr:class="textarea"
                    attr:id={move || index.get()}
                />
            </div>
        </li>
    }
}
