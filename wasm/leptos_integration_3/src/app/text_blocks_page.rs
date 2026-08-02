use crate::app::add_css_and_js;
use crate::app::js_value_parsing;
use leptos::logging::log;
use leptos::prelude::*;
use leptos::task::spawn_local;

use leptos_meta::*;

use js_sys::{Array, Function, Promise};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::window;

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
        spawn_local(async {
            match beg_js_to_work_the_worker(vec!["list_tables".to_string()]).await {
                Ok(val) => {
                    let text = js_sys::JSON::stringify(&val)
                        .unwrap_or_else(|_| JsValue::from("(unstringifiable)").into());
                    leptos::logging::log!(
                        "Worker response: {}",
                        text.as_string().unwrap_or_default()
                    );
                }
                Err(e) => leptos::logging::log!("Worker error: {:?}", e),
            }
        });
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
                        text=text_blocks.text
                    />
                </ForEnumerate>
            </ul>
            <div>
                "this is all of the textblocks combined:"
                <ForEnumerate
                    each=move || list.get()
                    key=|text_blocks| text_blocks.id
                    let(_, text_blocks)
                >
                    <span>
                        {move || text_blocks.text.0.get()}
                        <br/>
                    </span>
                </ForEnumerate>
            </div>
        </div>
    }
}

#[component]
fn TextArea(
    index: ReadSignal<usize>,
    text: (ReadSignal<String>, WriteSignal<String>),
) -> impl IntoView {
    let (get_text, set_text) = text;

    view! {
        <li class="text-container" data-id={move || index.get()}>
            <div class="drag-handle">"⠿"</div>
            <div class="text-input-container">
                <textarea
                    id={move || index.get()}
                    class="textarea"
                    on:input=move |ev| {
                        set_text.set(event_target_value(&ev));
                    }
                    placeholder="Type something..."
                ></textarea>
            </div>
        </li>
    }
}

#[wasm_bindgen(js_name = javascript_im_begging_you)]
extern "C" {
    fn javascript_im_begging_you(args: &JsValue) -> js_sys::Promise;
}

async fn beg_js_to_work_the_worker(args: Vec<String>) -> Result<JsValue, JsValue> {
    let arr = Array::new();
    for arg in &args {
        arr.push(&JsValue::from_str(arg));
    }
    let promise = javascript_im_begging_you(&arr);
    let result = JsFuture::from(promise).await?;
    Ok(result)
}
