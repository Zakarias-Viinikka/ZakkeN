use crate::markdown_conversion::{MarkdownConversion, MarkdownInputEvent};
use leptos::prelude::*;

fn push_log(log: RwSignal<Vec<String>>, line: String) {
    log.update(|l| {
        l.push(line);
        if l.len() > 30 {
            l.remove(0);
        }
    });
}

#[component]
pub fn AppMain() -> impl IntoView {
    let log = RwSignal::new(Vec::<String>::new());

    let title = ArcRwSignal::new("My page".to_string());
    let body = ArcRwSignal::new("some **bold** text".to_string());
    let fake_server_target = body.clone();

    let ghost = RwSignal::new(false);
    let focus_end = ArcRwSignal::new(None::<String>);
    let focus_start = ArcRwSignal::new(None::<String>);

    let on_input = Callback::new(move |e: MarkdownInputEvent| {
        push_log(
            log,
            format!(
                "input {}: {:?} -> {:?} (was_enter={}, cursor={})",
                e.block_id, e.old_text, e.new_text, e.was_enter, e.cursor_pos
            ),
        );
    });
    let on_delete_request = Callback::new(move |id: String| {
        push_log(log, format!("delete request {id}"));
    });
    let on_merge_with_above = Callback::new(move |(id, text): (String, String)| {
        push_log(log, format!("merge with above {id}: {text:?}"));
    });
    let on_split = Callback::new(move |(id, before, after): (String, String, String)| {
        push_log(log, format!("split {id}: {before:?} | {after:?}"));
    });
    let on_navigate = Callback::new(move |(id, down): (String, bool)| {
        push_log(log, format!("navigate {id} {}", if down { "down" } else { "up" }));
        match (id.as_str(), down) {
            ("b1", true) => focus_start.set(Some("b2".to_string())),
            ("b2", false) => focus_end.set(Some("b1".to_string())),
            _ => {}
        }
    });

    view! {
        <div class="container">
            <MarkdownConversion
                block_id="b1"
                content=title
                is_title=true
                placeholder="Untitled"
                focus_at_end=focus_end
                focus_at_start=focus_start
                on_input=on_input
                on_delete_request=on_delete_request
                on_merge_with_above=on_merge_with_above
                on_split=on_split
                on_navigate=on_navigate
            />
            <MarkdownConversion
                block_id="b2"
                content=body
                ghost=ghost
                focus_at_end=focus_end
                focus_at_start=focus_start
                on_input=on_input
                on_delete_request=on_delete_request
                on_merge_with_above=on_merge_with_above
                on_split=on_split
                on_navigate=on_navigate
            />
            <button on:click=move |_| {
                fake_server_target.set("changed by the server".to_string());
                push_log(log, "fake server update on b2".to_string());
            }>"Fake server update on block 2"</button>
            <button on:click=move |_| ghost.update(|g| *g = !*g)>
                "Toggle ghost (read-only) on block 2"
            </button>
            <ul>
                {move || {
                    log.get()
                        .into_iter()
                        .map(|l| view! { <li>{l}</li> })
                        .collect_view()
                }}
            </ul>
        </div>
    }
}