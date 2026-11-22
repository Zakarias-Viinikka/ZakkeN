use std::collections::HashMap;

use leptos::prelude::*;

mod cm;
mod edit;
mod keys;
mod markdown_conversion;
use cm::PageTitle;
use markdown_conversion::{MarkdownConversion, MarkdownInputEvent};

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

fn push(log: RwSignal<Vec<String>>, line: String) {
    log.update(|l| {
        l.push(line);
        if l.len() > 30 {
            l.remove(0);
        }
    });
}

#[component]
fn App() -> impl IntoView {
    let log = RwSignal::new(Vec::<String>::new());

    let b1 = ArcRwSignal::new("My page".to_string());
    let b2 = ArcRwSignal::new("see [[Cats|p1]] and **bold** and [[Gone|p9]]".to_string());
    let b3 = ArcRwSignal::new(
        "emoji 👨\u{200d}👩\u{200d}👧 😀 and a long paragraph that wraps over several lines so we can check line wrapping and the caret behaviour in a block with plenty of text in it."
            .to_string(),
    );
    let b2_server = b2.clone();

    let titles = RwSignal::new(HashMap::from([(
        "p1".to_string(),
        PageTitle { title: "Cats".to_string(), disabled: false },
    )]));
    let ghost = RwSignal::new(false);
    let focus_end = ArcRwSignal::new(None::<String>);
    let focus_start = ArcRwSignal::new(None::<String>);

    let on_input = Callback::new(move |e: MarkdownInputEvent| {
        push(
            log,
            format!(
                "input {}: {:?} -> {:?} (was_enter={}, cursor={})",
                e.block_id, e.old_text, e.new_text, e.was_enter, e.cursor_pos
            ),
        );
    });
    let on_delete_request = Callback::new(move |id: String| push(log, format!("delete request {id}")));
    let on_merge_with_above =
        Callback::new(move |(id, text): (String, String)| push(log, format!("merge with above {id}: {text:?}")));
    let on_split = Callback::new(move |(id, before, after): (String, String, String)| {
        push(log, format!("split {id}: {before:?} | {after:?}"));
    });
    let on_link_click = Callback::new(move |page_id: String| push(log, format!("link click {page_id}")));

    let fe = focus_end.clone();
    let fs = focus_start.clone();
    let on_navigate = Callback::new(move |(id, down): (String, bool)| {
        push(log, format!("navigate {id} {}", if down { "down" } else { "up" }));
        match (id.as_str(), down) {
            ("b1", true) => fs.set(Some("b2".to_string())),
            ("b2", true) => fs.set(Some("b3".to_string())),
            ("b2", false) => fe.set(Some("b1".to_string())),
            ("b3", false) => fe.set(Some("b2".to_string())),
            _ => {}
        }
    });

    view! {
        <h3>"markdown spike rewrite: final demo"</h3>
        <MarkdownConversion
            block_id="b1"
            content=b1
            is_title=true
            placeholder="Untitled"
            titles=titles
            focus_at_end=focus_end.clone()
            focus_at_start=focus_start.clone()
            on_input=on_input
            on_delete_request=on_delete_request
            on_merge_with_above=on_merge_with_above
            on_split=on_split
            on_navigate=on_navigate
            on_link_click=on_link_click
        />
        <MarkdownConversion
            block_id="b2"
            content=b2
            placeholder="Type '/' for commands..."
            titles=titles
            focus_at_end=focus_end.clone()
            focus_at_start=focus_start.clone()
            on_input=on_input
            on_delete_request=on_delete_request
            on_merge_with_above=on_merge_with_above
            on_split=on_split
            on_navigate=on_navigate
            on_link_click=on_link_click
        />
        <MarkdownConversion
            block_id="b3"
            content=b3
            ghost=ghost
            titles=titles
            focus_at_end=focus_end
            focus_at_start=focus_start
            on_input=on_input
            on_delete_request=on_delete_request
            on_merge_with_above=on_merge_with_above
            on_split=on_split
            on_navigate=on_navigate
            on_link_click=on_link_click
        />
        <button on:click=move |_| {
            b2_server.set("changed by the server, see [[Cats|p1]]".to_string());
            push(log, "fake server update on b2".to_string());
        }>"Fake server update on b2"</button>
        <button on:click=move |_| {
            titles.update(|t| {
                if let Some(e) = t.get_mut("p1") {
                    e.title = "Felines".to_string();
                }
            })
        }>"Rename page p1"</button>
        <button on:click=move |_| {
            titles.update(|t| {
                if let Some(e) = t.get_mut("p1") {
                    e.disabled = !e.disabled;
                }
            })
        }>"Toggle disabled p1"</button>
        <button on:click=move |_| ghost.update(|g| *g = !*g)>"Toggle ghost (read-only) on b3"</button>
        <ul>
            {move || log.get().into_iter().map(|l| view! { <li>{l}</li> }).collect_view()}
        </ul>
    }
}
