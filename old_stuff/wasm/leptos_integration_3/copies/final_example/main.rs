use leptos::logging::log;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_starter::final_example::js_stuff;
use leptos_starter::final_example::js_value_parsing;
use leptos_starter::javascript_take_the_wheel;

use leptos::task::spawn_local_scoped;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

#[derive(Copy, Clone)] // now Copy because signals are Copy
struct TextBlocks {
    // (getter, setter) tuple
    text: (ReadSignal<String>, WriteSignal<String>),
    id: usize,
}

impl TextBlocks {
    fn new(id: usize) -> Self {
        Self {
            text: signal(String::new()), // returns (ReadSignal, WriteSignal)
            id,
        }
    }
}

#[component]
fn App() -> impl IntoView {
    provide_meta_context();

    // list is now a read signal, set_list is the writer
    let (list, set_list) = signal(Vec::new());

    // populate the list
    set_list.update(|l| {
        for _ in 0..5 {
            l.push(TextBlocks::new(l.len()));
        }
    });

    // js handle
    javascript_take_the_wheel!("update_list_order", |js_value| {
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

    // Sortable detection – already uses modern signal()
    let (sortablejs_has_loaded, set_sortablejs_has_loaded) = signal(false);

    Effect::new(move |_| {
        if !sortablejs_has_loaded.get_untracked() {
            spawn_local_scoped(wait_for_sortable(set_sortablejs_has_loaded));
        }
    });

    view! {
        <Stylesheet href="/public/finale/finale.css"/>
        <Script src="/public/finale/Sortable.js"/>
        {move || {
            if sortablejs_has_loaded.get() {
                view! { <Script src="/public/finale/js.js"/> }.into_any()
            } else {
                view! { "" }.into_any()
            }
        }}

        <div class="finale-container">
            <ul id="sortable-container">
                 <ForEnumerate
                     each=move || list.get()
                     key=|text_blocks| text_blocks.id
                     let(index, text_blocks)
                >
                    <TextArea
                        index=index
                        text=text_blocks.text   // passing the (read, write) tuple
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
                    {move || text_blocks.text.0.get()}  // use the getter
                    <br/>
                </span>
             </ForEnumerate>
             </div>
        </div>
        <js_stuff::JsStuff />
    }
}

#[component]
fn TextArea(
    index: ReadSignal<usize>,
    text: (ReadSignal<String>, WriteSignal<String>), // tuple of getter/setter
) -> impl IntoView {
    let (get_text, set_text) = text; // destructure for convenience

    view! {
        <li class="text-container" data-id={move || index.get()}>
            <div class="drag-handle">"⠿"</div>
            <div class="text-input-container">
                <textarea
                    id={move || index.get()}
                    class="textarea"
                    on:input=move |ev| {
                        set_text.set(event_target_value(&ev));   // use setter
                    }
                    placeholder="Type something..."
                ></textarea>
            </div>
        </li>
    }
}

async fn wait_for_sortable(setter: WriteSignal<bool>) {
    loop {
        let ok = web_sys::window()
            .and_then(|w| w.get("Sortable"))
            .map(|_| true)
            .unwrap_or(false);
        if ok {
            setter.set(true);
            break;
        }
        gloo_timers::future::TimeoutFuture::new(50).await;
    }
}
