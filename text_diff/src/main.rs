use codee::string::FromToStringCodec;
use codee::string::JsonSerdeCodec;

use leptos::{prelude::*, svg::text, text_prop};
use leptos_use::storage::use_local_storage;

fn main() {
    console_error_panic_hook::set_once();
    //  trunk serve --open

    mount_to_body(App);
}

struct TextArea {
    getter: Signal<String>,
    setter: WriteSignal<String>,
    index: usize,
}

impl TextArea {
    fn new(index: usize, getter: Signal<String>, setter: WriteSignal<String>) -> Self {
        Self {
            getter,
            setter,
            index,
        }
    }
}

#[component]
fn App() -> impl IntoView {
    let nr_of_textareas = 5;
    let (latest_diff, latest_diff_set) = signal("".to_string());

    /*let (text_fields, set_text_fields, _) =
    use_local_storage::<Vec<String>, JsonSerdeCodec>("text_fields");*/
    let list: Vec<TextArea> = (0..nr_of_textareas)
        .map(|i| {
            let (getter, setter, _) = use_local_storage::<String, JsonSerdeCodec>(format!(
                "{}{}",
                nr_of_textareas,
                i.to_string()
            ));
            TextArea::new(i, getter, setter)
        })
        .collect();

    use leptos::html::Textarea;
    let list_of_node_refs: Vec<NodeRef<Textarea>> = (0..nr_of_textareas)
        .map(|_| NodeRef::<Textarea>::new())
        .collect();

    view! {
        /*
         * <ForEnumerate
             each=move || counters.get() // Same as <For/>
             key=|counter| counter.id    // Same as <For/>
             // Provides the index as a signal and the child T
             children={move |index: ReadSignal<usize>, counter: Counter| {
                 view! {
                     <button>{move || index.get()} ". Value: " {move || counter.count.get()}</button>
                 }
             }}
         />
         */
         <ForEnumerate
         each=move ||
         {
             (0..nr_of_textareas).map(|i| view! {
                 <textarea
                 on:beforeinput=move |ev| {

                 }
                 id=i></textarea><br/>
             })
             .collect_view()
         }


        <div id="latest-diff-result"> {latest_diff.get()}</div>
    }
}

fn get_diff(old: impl AsRef<str>, new: impl AsRef<str>) -> String {
    todo!()
}

fn set_text_field(
    setter: WriteSignal<String>,
    new_value: impl AsRef<str>,
    old_value: impl AsRef<str>,
) {
    setter.set(new_value.as_ref().into());
}
fn read_text_field(getter: ReadSignal<Vec<String>, JsonSerdeCodec>) -> String {
    todo!()
}
